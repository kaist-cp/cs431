//! Modules `boc_fibonacci` and `boc_banking` are taken from <https://github.com/ic-slurp/verona-benchmarks/tree/main/savina/boc>.

mod boc_fibonacci {
    //! Computing fibonacci sequence using [`boc`].

    use crossbeam_channel::{bounded, Sender};
    use cs431_homework::boc::run_when;
    use cs431_homework::{tuple_list, when, CownPtr};

    fn fibonacci_inner(n: usize, sender: Option<Sender<usize>>) -> CownPtr<usize> {
        if n == 0 {
            CownPtr::new(0)
        } else if n == 1 {
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
        } else if n == 1 {
            return 1;
        }

        let (finish_sender, finish_receiver) = bounded(0);
        let _ = fibonacci_inner(n, Some(finish_sender));

        finish_receiver.recv().unwrap()
    }
}

mod boc_banking {
    //! A simple transaction system using [`boc`].

    use std::thread::sleep;
    use std::time::Duration;

    use crossbeam_channel::bounded;
    use cs431_homework::boc::run_when;
    use cs431_homework::test::RandGen;
    use cs431_homework::{tuple_list, when, CownPtr};
    use rand::thread_rng;

    const TRANSFER_LIMIT: usize = 2048;

    pub fn run_transactions(account_cnt: usize, transaction_cnt: usize, use_sleep: bool) {
        assert_ne!(account_cnt, 0);
        assert_ne!(transaction_cnt, 0);

        let mut rng = thread_rng();
        let accounts: Box<[_]> = (0..account_cnt)
            .map(|_| CownPtr::new(usize::rand_gen(&mut rng)))
            .collect();

        let c_remaining = CownPtr::new(transaction_cnt);

        let (finish_sender, finish_receiver) = bounded(0);

        let mut rng = thread_rng();
        for _ in 0..transaction_cnt {
            // Randomly pick src and dest accounts.
            let src = usize::rand_gen(&mut rng) % account_cnt;
            let mut dst = usize::rand_gen(&mut rng) % account_cnt;

            if src == dst {
                dst = (dst + 1) % account_cnt;
            }

            let amount = usize::rand_gen(&mut rng) % TRANSFER_LIMIT;
            let random_sleep = use_sleep && usize::rand_gen(&mut rng) % 2 == 0;

            let c_src = accounts[src].clone();
            let c_dst = accounts[dst].clone();

            // FIXME: This clone and the clone in the lower `when!` seems stupid but is needed.
            let finish_sender = finish_sender.clone();
            let c_remaining = c_remaining.clone();
            when!(c_src, c_dst; src, dst; {
                // Transfer.
                if amount <= *src {
                    *src -= amount;
                    *dst += amount;
                }

                if random_sleep {
                    sleep(Duration::from_secs(1));
                }

                let finish_sender = finish_sender.clone();
                when!(c_remaining; remaining; {
                    *remaining -= 1;
                    // Tell the main thread that all transactions have finished.
                    if *remaining == 0 {
                        finish_sender.send(()).unwrap();
                    }
                });
            });
        }

        finish_receiver.recv().unwrap();
    }
}

mod boc_merge_sort {
    //! Merge sort using BoC.

    use crossbeam_channel::{bounded, Sender};
    use cs431_homework::boc::{run_when, CownPtr};
    use cs431_homework::{tuple_list, when};

    fn merge_sort_inner(
        idx: usize,
        step_size: usize,
        n: usize,
        boc_arr: &[CownPtr<usize>],
        boc_finish: &[CownPtr<usize>],
        sender: &Sender<Vec<usize>>,
    ) {
        if idx == 0 {
            return;
        }

        // Recursively sort a subarray within range [from, to).
        let from = idx * step_size - n;
        let to = (idx + 1) * step_size - n;

        let mut bocs = boc_arr[from..to].to_vec();
        bocs.push(boc_finish[idx].clone());
        bocs.push(boc_finish[idx * 2].clone());
        bocs.push(boc_finish[idx * 2 + 1].clone());

        let boc_arr: Box<[_]> = boc_arr.into();
        let boc_finish: Box<[_]> = boc_finish.into();
        let sender = sender.clone();

        run_when(bocs, move |mut content| {
            let left_and_right_sorted =
                (*content[step_size + 1] == 1) && (*content[step_size + 2] == 1);
            if !left_and_right_sorted || *content[step_size] == 1 {
                // If both subarrays are not ready or we already sorted for this range, skip.
                return;
            }

            // Now, merge the two subarrays.
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

            // Signal that we have sorted the subarray [from, to).
            *content[step_size] = 1;

            // If the sorting process is completed send a signal to the main thread.
            if idx == 1 {
                sender.send(res).unwrap();
                return;
            }

            // Recursively sort the larger subarray (bottom up)
            merge_sort_inner(idx / 2, step_size * 2, n, &boc_arr, &boc_finish, &sender);
        });
    }

    /// Sorts and returns a sorted version of `array`.
    // TODO: We could make this generic over `T : Ord + Send`, but it might also need `Default` or
    // usage of `MabyeUninit`.
    pub fn merge_sort(array: Vec<usize>) -> Vec<usize> {
        let n = array.len();
        if n == 1 {
            return array;
        }

        let boc_arr: Box<[CownPtr<usize>]> = array.into_iter().map(CownPtr::new).collect();
        let boc_finish: Box<[CownPtr<usize>]> = (0..(2 * n)).map(|_| CownPtr::new(0)).collect();

        let (finish_sender, finish_receiver) = bounded(0);

        for i in 0..n {
            let c_finished = boc_finish[i + n].clone();

            let boc_arr_clone = boc_arr.clone();
            let boc_finish_clone = boc_finish.clone();
            let finish_sender = finish_sender.clone();
            when!(c_finished; finished; {
                // Signal that the sorting of subarray for [i, i+1) is finished.
                *finished = 1;
                merge_sort_inner((n + i) / 2, 2, n, &boc_arr_clone, &boc_finish_clone, &finish_sender);
            });
        }

        // Wait until sorting finishes and get the result.
        finish_receiver.recv().unwrap()
    }
}

mod basic_test {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use crossbeam_channel::bounded;
    use cs431_homework::boc::{run_when, CownPtr};
    use cs431_homework::{tuple_list, when};

    use crate::{boc_banking, boc_fibonacci, boc_merge_sort};

    #[test]
    fn message_passing() {
        for _ in 0..20 {
            let c_sent_t1 = CownPtr::new(false);
            let c_sent_t2 = c_sent_t1.clone();
            let msg_t1 = Arc::new(AtomicUsize::new(0));
            let msg_t2 = msg_t1.clone();
            let msg_t3 = msg_t1.clone();

            let (send1, recv1) = bounded(1);
            let (send2, recv2) = bounded(1);

            rayon::spawn(move || {
                when!(c_sent_t1; sent; {
                    if !*sent {
                        msg_t1.fetch_add(1, Ordering::Relaxed);
                        *sent = true;
                    } else {
                        assert_eq!(1, msg_t1.load(Ordering::Relaxed));
                    }
                    send1.send(()).unwrap();
                });
            });
            rayon::spawn(move || {
                when!(c_sent_t2; sent; {
                    if !*sent {
                        msg_t2.fetch_add(1, Ordering::Relaxed);
                        *sent = true;
                    } else {
                        assert_eq!(1, msg_t2.load(Ordering::Relaxed));
                    }
                    send2.send(()).unwrap();
                });
            });

            recv1.recv().unwrap();
            recv2.recv().unwrap();

            assert_eq!(1, msg_t3.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn message_passing_determines_order() {
        for _ in 0..20 {
            let c_flag1_t1 = CownPtr::new(false);
            let c_flag2_t1 = CownPtr::new(false);
            let c_flag1_t2 = c_flag1_t1.clone();
            let c_flag2_t2 = c_flag2_t1.clone();
            let msg_t1 = Arc::new(AtomicUsize::new(0));
            let msg_t2 = msg_t1.clone();
            let msg_t3 = msg_t1.clone();

            let (send_t1, recv) = bounded(0);
            let send_t2 = send_t1.clone();

            rayon::spawn(move || {
                when!(c_flag1_t1; flag1; *flag1 = true);

                if msg_t1
                    .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_err()
                {
                    when!(c_flag2_t1; flag2; {
                        assert!(*flag2);
                        send_t1.send(()).unwrap();
                    });
                }
            });
            rayon::spawn(move || {
                when!(c_flag2_t2; flag2; *flag2 = true);

                if msg_t2
                    .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_err()
                {
                    when!(c_flag1_t2; flag1; {
                        assert!(*flag1);
                        send_t2.send(()).unwrap();
                    });
                }
            });

            recv.recv().unwrap();

            assert_eq!(1, msg_t3.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn fibonacci() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            let mut accumulator = vec![0, 1];
            for n in 2..=25 {
                let answer = accumulator[n - 2] + accumulator[n - 1];
                accumulator.push(answer);
                assert_eq!(answer, boc_fibonacci::fibonacci(n));
            }

            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }

    #[test]
    fn merge_sort() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            let mut arr1 = vec![2, 3, 1, 4];
            let res1 = boc_merge_sort::merge_sort(arr1.clone());
            arr1.sort();
            assert_eq!(arr1, res1);

            let mut arr2 = vec![3, 4, 2, 1, 8, 5, 6, 7];
            let res2 = boc_merge_sort::merge_sort(arr2.clone());
            arr2.sort();
            assert_eq!(arr2, res2);

            let res2_ = boc_merge_sort::merge_sort(arr2.clone());
            assert_eq!(arr2, res2_);

            let mut arr3 = arr2.clone();
            arr3.append(&mut arr3.clone());
            arr3.append(&mut arr3.clone());
            arr3.append(&mut arr3.clone());
            arr3.append(&mut arr3.clone());
            arr3.append(&mut arr3.clone());
            let res3 = boc_merge_sort::merge_sort(arr3.clone());
            arr3.sort();
            assert_eq!(arr3, res3);

            let mut arr4: Vec<_> = (0..1024).rev().collect();
            let res4 = boc_merge_sort::merge_sort(arr4.clone());
            arr4.sort();
            assert_eq!(arr4, res4);

            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }

    #[test]
    fn banking() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            boc_banking::run_transactions(20, 20, true);
            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }
}

mod stress_test {
    use crossbeam_channel::{bounded, Receiver, Sender};
    use cs431_homework::test::RandGen;
    use rand::thread_rng;

    use crate::{boc_banking, boc_fibonacci, boc_merge_sort};

    #[test]
    fn fibonacci() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            assert_eq!(boc_fibonacci::fibonacci(28), 317811);
            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }

    #[test]
    fn banking() {
        let (send_finish, recv_finish) = bounded(0);

        rayon::spawn(move || {
            boc_banking::run_transactions(1234, 100000, false);
            send_finish.send(()).unwrap();
        });

        recv_finish.recv().unwrap();
    }

    #[test]
    fn merge_sort() {
        const ITER: usize = 4;
        const LOGSZ_LO: usize = 10;
        const LOGSZ_HI: usize = 13;

        let (senders, receivers): (Vec<Sender<()>>, Vec<Receiver<()>>) =
            (0..ITER).map(|_| bounded(1)).unzip();
        let mut rng = thread_rng();

        for (i, sender) in senders.into_iter().enumerate() {
            let logsize = LOGSZ_LO + i % (LOGSZ_HI - LOGSZ_LO);
            let len = 1 << logsize;
            let mut arr: Vec<_> = (0..len).map(|_| usize::rand_gen(&mut rng)).collect();
            rayon::spawn(move || {
                let res = boc_merge_sort::merge_sort(arr.clone());
                arr.sort_unstable();
                assert_eq!(arr, res);
                sender.send(()).unwrap();
            });
        }

        for reciever in receivers {
            reciever.recv().unwrap();
        }
    }
}
