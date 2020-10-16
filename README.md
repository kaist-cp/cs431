# KAIST CS492: Design and Analysis of Concurrent Programs

## Logistics

- Instructor: [Jeehoon Kang](https://cp.kaist.ac.kr/jeehoon.kang)
- TA: [Jaehwang Jung](https://cp.kaist.ac.kr/jaehwang.jung)
    + Office Hour: Friday 13:30-16:30 [(Google Meet)](https://meet.google.com/ftr-epmw-pmv).
      See [below](#communication) for the office hour policy.
- Time & Place: Tue & Thu 10:30am-11:45am, ~~Rm 1101, Bldg E3-1~~ [Youtube channel](https://www.youtube.com/playlist?list=PL5aMzERQ_OZ9j40DJNlsem2qAGoFbfwb4), [Google Meet chatting room](https://meet.google.com/zqd-qmgd-sga)
- Website: https://github.com/kaist-cp/cs492-concur
- Announcements: in [issue
  tracker](https://github.com/kaist-cp/cs492-concur/issues?q=is%3Aissue+is%3Aopen+label%3Aannouncement)



### Online sessions

Due to COVID-19, we're going to conduct online sessions for this semester.

- For non-live sessions, videos will be uploaded to a [Youtube channel](https://www.youtube.com/playlist?list=PL5aMzERQ_OZ9j40DJNlsem2qAGoFbfwb4).

    + You're required to watch the video, and based on the contents, to solve pop quizzes that will
      be posted at gg.kaist.ac.kr. The details are announced in [an
      issue](https://github.com/kaist-cp/cs492-concur/issues/42).

    + Turn on English subtitle at YouTube, if necessary.

- For live sessions, we'll meet in a [Google Meet chatting room](https://meet.google.com/zqd-qmgd-sga).



## Course description

### Context

I expect computers in the next 700 years will be **massively parallel**. We the humankind want to
improve the performance of computers in the era of big data. But it is becoming more and more
challenging after the breakdown of [Dennard scaling](https://en.wikipedia.org/wiki/Dennard_scaling)
around 2005, which means the performance of sequential computers will not be improved. Thus not only
servers but also personal computers have been multi-core systems since then. The problem is only
worsened by the ending of the [Moore's law](https://en.wikipedia.org/wiki/Moore%27s_law), which
means we are no longer able to benefit from denser electrical circuit. It seems the only remaining
way to optimize performance is specialization, which aims to better exploit parallelism of
workloads. Because of these technology trends, I expect computers in the future will be massively
parallel.

But we are not ready yet for the era of massive parallelism. The main difficulty lies on handling
**shared mutable states**, which is the main topic of concurrency. To coordinate multiple cores and
other resources, their inputs and outputs should be somehow properly synchronized with each other
via shared mutable states like memory. But handling shared mutable states is inherently challenging,
both theoretically and practically. For example, in the presence of thousands and millions of cores,
how to efficiently synchronize concurrent accesses to shared memory? In the presence of
nondeterministic interleaving of thread executions, how to make sure the safety of a concurrent
program? In the presence of compiler and hardware optimizations, what is the right specification of
a concurrent data structure?

Fortunately, the theory of shared mutable states has advanced quite impressively in the past five
years, and now it is greatly helpful in designing and analyzing practical systems with shared
mutable states. So in this course, we will discuss the recent theory of shared mutable states and
its application to real-world practical systems.


### Goal

This course is geared towards senior undergraduate/graduate students in computer science (or related
disciplines) who are interested in the modern theory and practice of parallel computer systems.
This course aims to help such students to:

- Understand the motivations and challenges in concurrent programming
- Learn design patterns and reasoning principles of concurrent programs
- Design, implement, and evaluate concurrent programs
- Apply the understanding to real-world parallel systems


### Textbook

- [Slides](https://docs.google.com/presentation/d/1NMg08N1LUNDPuMxNZ-UMbdH13p8LXgMM3esbWRMowhU/edit?usp=sharing)
- Classical papers and libraries
    + [Promising semantics](https://sf.snu.ac.kr/promise-concurrency/): programming language &
      architecture semantics for shared-memory concurrency
    + [Crossbeam](https://github.com/crossbeam-rs/crossbeam): concurrent data structure library in
      [Rust](https://www.rust-lang.org/)


### Tools

Make sure you're capable of using the following development tools:

- [Git](https://git-scm.com/): for downloading the homework skeleton and version-controlling your
  development. If you're not familiar with Git, walk through [this
  tutorial](https://www.atlassian.com/git/tutorials).

    + **IMPORTANT**: you should not expose your work to others. In particular, you should not fork
      the [upstream](https://github.com/kaist-cp/cs492-concur) and push there. Please the following
      steps:

        * Directly clone the upstream without forking it.

          ```bash
          $ git clone --origin upstream https://github.com/kaist-cp/cs492-concur.git
          $ cd cs492-concur
          $ git remote -v
          upstream	https://github.com/kaist-cp/cs492-concur.git (fetch)
          upstream	https://github.com/kaist-cp/cs492-concur.git (push)
          ```

        * To get updates from the upstream, fetch and merge `upstream/master`.

          ```bash
          $ git fetch upstream
          $ git merge upstream/master
          ```

    + If you want to manage your development in a Git server, please create your own private
      repository.

        * You may upgrade your GitHub account to "PRO", which is free of charge.  Refer to the
          [documentation](https://education.github.com/students)

        * Set up your repository as a remote.

          ```bash
          $ git remote add origin git@github.com:<github-id>/cs492-concur.git
          $ git remote -v
          origin	 git@github.com:<github-id>/cs492-concur.git (fetch)
          origin	 git@github.com:<github-id>/cs492-concur.git (push)
          upstream https://github.com/kaist-cp/cs492-concur.git (fetch)
          upstream https://github.com/kaist-cp/cs492-concur.git (push)
          ```

        * Push to your repository.

          ```bash
          $ git push -u origin master
          ```

- [Rust](https://www.rust-lang.org/): as the language of homework implementation. We chose Rust
  because its ownership type system greatly simplifies the development of large-scale system
  software.

  We recommend you to read [this page](https://github.com/kaist-cp/helpdesk/#specialty) that
  describes how to study Rust.

- [Visual Studio Code](https://code.visualstudio.com/) (optional): for developing your homework. If
  you prefer other editors, you're good to go.
      
- You can connect to server by `ssh s<student-id>@cp-service.kaist.ac.kr -p14000`, e.g., `ssh
  s20071163@cp-service.kaist.ac.kr -p14000`. See [this
  issue](https://github.com/kaist-cp/cs492-concur/issues/42) for more detail.

    + Add the following lines in your `~/.ssh/config`:
    
      ```
      Host cs492-concur
        Hostname cp-service.kaist.ac.kr
        Port 14000
        User s<student-id>
      ```
      
      Then you can connect to the server by `ssh cs492-concur`.

    + Now you can [use it as a VSCode remote server as in the video](https://www.youtube.com/watch?v=TTVuUIhdn_g&list=PL5aMzERQ_OZ8RWqn-XiZLXm1IJuaQbXp0&index=3).



## Prerequisites

- It is **strongly recommended** that students already took courses on:

    + Mathematics (freshman calculus, MAS101 & MAS102): proposition statement and proof
    + Data structures (CS206): linked list, stack, queue
    + Systems programming (CS230): memory layout, cache, lock
    + Programming languages (CS320): lambda calculus, interpreter

  Without a proper understanding of these topics, you will likely struggle in this course.

- Other recommendations which would help you in this course:

    + Basic understanding of computer architecture (CS311)
    + Programming experience in [Rust](https://www.rust-lang.org/)



## Grading & honor code

### Homework & project (60%)

- Reading assignment (both paper and code)
- Programming assignment (including design, implementation, evaluation)

### Midterm and final exams (40%)

The exams will evaluate your theoretical understanding of shared mutable states.

### Attendance (?%)

You should submit a token to the [Course Management](https://gg.kaist.ac.kr/course/6) website for
each session.  You should submit a token within **12 hours from the beginning of a sessions**.

### Honor code

[Please sign KAIST School of Computing Honor Code here](https://gg.kaist.ac.kr/quiz/17/).



## Communication

- Course-related announcements and information will be posted on the
  [website](https://github.com/kaist-cp/cs492-concur) as well as on the [GitHub issue
  tracker](https://github.com/kaist-cp/cs492-concur/issues).  You are expected to read all
  announcements within 24 hours of their being posted.  It is highly recommended to watch the
  repository so that new announcements will automatically be delivered to you email address.

- Ask your questions via email **only if** they are either confidential or personal.  Otherwise, ask
   questions in [this repository's issue tracker](https://github.com/kaist-cp/cs492-concur/issues).
   Any questions failing to do so (e.g. email questions on course materials) will not be answered.

    + I'm requiring you to ask questions online first for two reasons. First, clearly writing a
      question is the first step to reach an answer. Second, you can benefit from questions and
      answers of other students.

- We are NOT going to discuss *new* questions during the office hour. Before coming to the office
  hour, please check if there is a similar question on the issue tracker.  If there isn't, file a
  new issue and start discussion there. The agenda of the office hour will be the issues that are
  not resolved yet.

- Emails to the instructor or TAs should begin with "CS492C:" in the subject line, followed by a
  brief description of the purpose of your email.  The content should at least contain your name and
  student number.  Any emails failing to do so (e.g. emails without student number) will not be
  answered.
