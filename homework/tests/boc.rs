// Two modules `boc_fibonacci` and `boc_banking` are from https://github.com/ic-slurp/verona-benchmarks/tree/main/savina/boc

/// Implementation of computing fibonacci sequence
mod boc_fibonacci {
    use crossbeam_channel::bounded;
    use cs431_homework::{boc::run_when, tuple_list, when, CownPtr};

    fn fibonacci_inner(
        n: usize,
        sender: Option<crossbeam_channel::Sender<usize>>,
    ) -> CownPtr<usize> {
        if n == 0 {
            CownPtr::new(0)
        } else if n <= 2 {
            CownPtr::new(1)
        } else {
            let prev = fibonacci_inner(n - 1, None);
            let pprev = fibonacci_inner(n - 2, None);
            when!(prev, pprev; g1, g2; {
                *g1 += *g2;
                if let Some(sender) = &sender {
                    sender.send(*g1).unwrap();
                }
            });
            return prev;
        }
    }

    pub fn fibonacci(n: usize) -> usize {
        if n == 0 {
            return 0;
        } else if n <= 2 {
            return 1;
        }

        let (finish_sender, finish_receiver) = bounded(0);
        let _ = fibonacci_inner(n, Some(finish_sender));

        finish_receiver.recv().unwrap()
    }
}

mod boc_banking {
    use std::{thread::sleep, time::Duration};

    use crossbeam_channel::bounded;
    use cs431_homework::{boc::run_when, test::RandGen, tuple_list, when, CownPtr};
    use rand::thread_rng;

    const TRANSFER_LIMIT: usize = 2048;

    pub fn run_transactions(account_cnt: usize, transaction_cnt: usize, use_sleep: bool) {
        assert_ne!(account_cnt, 0);
        assert_ne!(transaction_cnt, 0);

        let mut rng = thread_rng();
        let accounts: Vec<CownPtr<usize>> = (0..account_cnt)
            .map(|_| CownPtr::new(usize::rand_gen(&mut rng)))
            .collect();
        let teller: CownPtr<(Vec<CownPtr<usize>>, usize)> =
            CownPtr::new((accounts, transaction_cnt));

        let (finish_sender, finish_receiver) = bounded(0);

        when!(teller; teller_inner; {
            let mut rng = thread_rng();
            for _ in 0..transaction_cnt {
                // randomly pick src and dest accounts
                let src = usize::rand_gen(&mut rng) % account_cnt;
                let mut dst = usize::rand_gen(&mut rng) % account_cnt;
                if src == dst { dst = (dst + 1) % account_cnt; }

                let amount = usize::rand_gen(&mut rng) % TRANSFER_LIMIT;
                let random_sleep = usize::rand_gen(&mut rng) % 2 == 0;

                let cown1 = teller_inner.0[src].clone();
                let cown2 = teller_inner.0[dst].clone();
                let teller = teller.clone();
                let finish_sender = finish_sender.clone();

                when!(cown1, cown2; g1, g2; {
                    // transfer
                    if amount <= *g1 { *g1 -= amount; *g2 += amount; }
                    if random_sleep && use_sleep { sleep(Duration::from_secs(1)); }

                    let finish_sender = finish_sender.clone();

                    // Main thread waits until all transactions finish
                    when!(teller; teller_inner; {
                        teller_inner.1 -= 1;
                        if teller_inner.1 == 0 {
                            finish_sender.send(()).unwrap();
                        }
                    });
                });
            }
        });

        finish_receiver.recv().unwrap();
    }
}

/// Implementation of a merge sort that uses BoC
mod boc_merge_sort {
    use crossbeam_channel::bounded;
    use cs431_homework::{
        boc::{run_when, CownPtr},
        tuple_list, when,
    };

    fn merge_sort_inner(
        idx: usize,
        step_size: usize,
        n: usize,
        boc_arr: &Vec<CownPtr<usize>>,
        boc_finish: &Vec<CownPtr<usize>>,
        sender: &crossbeam_channel::Sender<Vec<usize>>,
    ) {
        if idx == 0 {
            return;
        }

        // Recursively sort a subarray within range [from, to)
        let from = idx * step_size - n;
        let to = (idx + 1) * step_size - n;

        let mut bocs: Vec<CownPtr<usize>> = boc_arr[from..to].iter().map(|x| x.clone()).collect();
        bocs.push(boc_finish[idx].clone());
        bocs.push(boc_finish[idx * 2].clone());
        bocs.push(boc_finish[idx * 2 + 1].clone());

        let boc_arr_clone = boc_arr.clone();
        let boc_finish_clone = boc_finish.clone();
        let sender_clone = sender.clone();

        run_when(bocs, move |mut content| {
            // Check if both left and right subarrays are already sorted
            let ready = (*content[step_size + 1] == 1) && (*content[step_size + 2] == 1);
            if !ready || *content[step_size] == 1 {
                return; // We skip if both subarrays are not ready or we already sorted for this range
            }

            // Now, merge the two subarrays
            let mut lo = 0;
            let mut hi = step_size / 2;
            let mut res = Vec::new();
            while res.len() < step_size {
                if lo >= step_size / 2 || (hi < step_size && *content[lo] > *content[hi]) {
                    res.push(*content[hi]);
                    hi += 1;
                } else {
                    res.push(*content[lo]);
                    lo += 1;
                }
            }
            for i in 0..step_size {
                *content[i] = res[i];
            }

            // Signal we have sorted the subarray within range [from, to)
            *content[step_size] = 1;

            // Send a signal to main thread if this completes the sorting process
            if idx == 1 {
                sender_clone.send(res).unwrap();
                return;
            }

            // Recursively sort the larger subarray (bottom up)
            merge_sort_inner(
                idx / 2,
                step_size * 2,
                n,
                &boc_arr_clone,
                &boc_finish_clone,
                &sender_clone,
            );
        });
    }

    /// The main function of merge sort that returns the sorted array of `arr`
    /// Assumption: `arr` should have size of 2^`logsize`
    pub fn merge_sort(arr: Vec<usize>, logsize: usize) -> Vec<usize> {
        let n: usize = 1 << logsize;
        assert_eq!(arr.len(), n);
        if logsize == 0 {
            return arr;
        }

        let boc_arr: Vec<CownPtr<usize>> = arr.iter().map(|x| CownPtr::new(*x)).collect();
        let boc_finish: Vec<CownPtr<usize>> = (0..(2 * n)).map(|_| CownPtr::new(0)).collect();

        let (finish_sender, finish_receiver) = bounded(0);

        for i in 0..n {
            let arr_elem = boc_arr[i].clone();
            let finish_elem = boc_finish[i + n].clone();
            let boc_arr_clone = boc_arr.clone();
            let boc_finish_clone = boc_finish.clone();
            let sender = finish_sender.clone();
            when!(arr_elem, finish_elem; _garr, gfinish; {
                *gfinish = 1; // signals finish of sorting of subarray within range [i, i+1)
                merge_sort_inner((n + i) / 2, 2, n, &boc_arr_clone, &boc_finish_clone, &sender);
            });
        }

        // Wait until sorting finishes and get the result
        finish_receiver.recv().unwrap()
    }
}

mod basic_test {
    use crate::{boc_banking, boc_fibonacci, boc_merge_sort};
    use crossbeam_channel::bounded;
    use cs431_homework::{
        boc::{run_when, CownPtr},
        tuple_list, when,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn message_passing_test() {
        for _ in 0..20 {
            let c1 = CownPtr::new(false);
            let c1_ = c1.clone();
            let msg = Arc::new(AtomicUsize::new(0));
            let msg_ = msg.clone();
            let msg__ = msg.clone();

            let (send1, recv1) = bounded(1);
            let (send2, recv2) = bounded(1);

            rayon::spawn(move || {
                when!(c1; g1; {
                    if !*g1 {
                        msg.fetch_add(1, Ordering::Relaxed);
                        *g1 = true;
                    } else {
                        assert_eq!(1, msg.load(Ordering::Relaxed));
                    }
                    send1.send(()).unwrap();
                });
            });
            rayon::spawn(move || {
                when!(c1_; g1; {
                    if !*g1 {
                        msg_.fetch_add(1, Ordering::Relaxed);
                        *g1 = true;
                    } else {
                        assert_eq!(1, msg_.load(Ordering::Relaxed));
                    }
                    send2.send(()).unwrap();
                });
            });

            recv1.recv().unwrap();
            recv2.recv().unwrap();

            assert_eq!(1, msg__.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn message_passing_determines_order() {
        for _ in 0..20 {
            let c1 = CownPtr::new(false);
            let c2 = CownPtr::new(false);
            let c1_ = c1.clone();
            let c2_ = c2.clone();
            let msg = Arc::new(AtomicUsize::new(0));
            let msg_ = msg.clone();
            let msg__ = msg.clone();

            let (send1, recv) = bounded(0);
            let send2 = send1.clone();

            rayon::spawn(move || {
                when!(c1; g1; *g1 = true);
                if msg
                    .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_err()
                {
                    when!(c2; g2; {
                        assert!(*g2);
                        send1.send(()).unwrap();
                    });
                }
            });
            rayon::spawn(move || {
                when!(c2_; g2; *g2 = true);
                if msg_
                    .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_err()
                {
                    when!(c1_; g1; {
                        assert!(*g1);
                        send2.send(()).unwrap();
                    });
                }
            });

            recv.recv().unwrap();

            assert_eq!(1, msg__.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn fibonacci_basic_test() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            let mut arr = vec![0, 1, 1];
            while arr.len() <= 25 {
                let n = arr.len();
                let ans = arr[n - 2] + arr[n - 1];
                arr.push(ans);
                assert_eq!(ans, boc_fibonacci::fibonacci(n));
            }

            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }

    #[test]
    fn merge_sort_basic_test() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            let mut arr1 = vec![2, 3, 1, 4];
            let res1 = boc_merge_sort::merge_sort(arr1.clone(), 2);
            arr1.sort();
            assert_eq!(arr1, res1);

            let mut arr2 = vec![3, 4, 2, 1, 8, 5, 6, 7];
            let res2 = boc_merge_sort::merge_sort(arr2.clone(), 3);
            arr2.sort();
            assert_eq!(arr2, res2);

            let res2_ = boc_merge_sort::merge_sort(arr2.clone(), 3);
            assert_eq!(arr2, res2_);

            let mut arr3 = arr2.clone();
            arr3.append(&mut arr3.clone());
            arr3.append(&mut arr3.clone());
            arr3.append(&mut arr3.clone());
            arr3.append(&mut arr3.clone());
            arr3.append(&mut arr3.clone());
            let res3 = boc_merge_sort::merge_sort(arr3.clone(), 8);
            arr3.sort();
            assert_eq!(arr3, res3);

            let mut arr4: Vec<_> = (0..1024).rev().collect();
            let res4 = boc_merge_sort::merge_sort(arr4.clone(), 10);
            arr4.sort();
            assert_eq!(arr4, res4);

            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }

    #[test]
    fn banking_basic_test() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            boc_banking::run_transactions(20, 20, true);
            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }
}

mod stress_test {
    use crate::{boc_banking, boc_fibonacci, boc_merge_sort};
    use crossbeam_channel::bounded;
    use cs431_homework::test::RandGen;
    use rand::thread_rng;

    #[test]
    fn fibonacci_stress_test() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            assert_eq!(boc_fibonacci::fibonacci(32), 2178309);
            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }

    #[test]
    fn banking_stress_test() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            boc_banking::run_transactions(1234, 100000, false);
            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }

    #[test]
    fn merge_sort_stress_test() {
        const ITER: usize = 10;
        const LOGSZ_LO: usize = 10;
        const LOGSZ_HI: usize = 14;

        let channels: Vec<_> = (0..ITER).map(|_| bounded(1)).collect();
        let mut rng = thread_rng();

        for i in 0..ITER {
            let sender = channels[i].0.clone();
            let logsize = LOGSZ_LO + i % (LOGSZ_HI - LOGSZ_LO);
            let len = 1 << logsize;
            let mut arr: Vec<_> = (0..len).map(|_| usize::rand_gen(&mut rng)).collect();
            rayon::spawn(move || {
                let res = boc_merge_sort::merge_sort(arr.clone(), logsize);
                arr.sort();
                assert_eq!(arr, res);
                sender.send(()).unwrap();
            });
        }

        for i in 0..ITER {
            channels[i].1.recv().unwrap();
        }
    }
}
