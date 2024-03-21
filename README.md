# KAIST CS431: Concurrent Programming

## Logistics

- Instructor: [Jeehoon Kang](https://www.fearless.systems/jeehoon.kang)
- Time: Mon & Wed 13:00-14:15 (2024 Spring)
- Place
  + Rm. 1101, Bldg. E3-1. **YOUR PHYSICAL ATTENDANCE IS REQUIRED** unless announced otherwise.
  + [Zoom room](https://kaist.zoom.us/my/jeehoon.kang) (if remote participation is absolutely necessary).
    The passcode is announced at KLMS.
  + [Youtube channel](https://www.youtube.com/playlist?list=PL5aMzERQ_OZ9j40DJNlsem2qAGoFbfwb4).
    Turn on English subtitles on YouTube, if necessary.
- Websites: <https://github.com/kaist-cp/cs431>, <https://gg.kaist.ac.kr/course/19>
- Announcements: in the [issue tracker](https://github.com/kaist-cp/cs431/issues?q=is%3Aissue+is%3Aopen+label%3Aannouncement)
  + We assume that you will read each announcement within 24 hours.
  + We strongly recommend you watch the repository.
- TA: [Sunho Park](https://www.fearless.systems/sunho.park/) (Head TA), [Janggun Lee](https://www.fearless.systems/janggun.lee/).
  + Office Hours: Fri 9:15-10:15, Rm. 4432, Bldg. E3-1.
    If you want to come, do so by 9:30.
    See [below](https://github.com/kaist-cp/cs431#rules) for the office hour policy.
    <!-- Fri 9:00-12:00, [Zoom room](https://zoom.us/j/4842624821)(The passcode is same as the class). It is not required, but if you want to come, do so by 9:30. See [below](#communication) for office hour policy. -->
- **IMPORTANT**: you should not expose your work to others. In particular, you should not fork the [upstream](https://github.com/kaist-cp/cs431) and push there.


## Course description

### Context

I anticipate that in the next 700 years, computers will be **massively parallel**.
Humankind seeks to enhance computer performance in the era of big data.
This goal has become increasingly challenging following the breakdown of [Dennard scaling](https://en.wikipedia.org/wiki/Dennard_scaling) around 2005, indicating that the performance of sequential computers is unlikely to improve further.
Consequently, both servers and personal computers have adopted multi-core systems.
This challenge is compounded by the end of [Moore's Law](https://en.wikipedia.org/wiki/Moore%27s_law), which signifies our diminishing ability to benefit from denser electronic circuits.
It appears that the primary pathway to optimizing performance now lies in specialization, focusing on exploiting the parallelism in workloads.
Due to these technological trends, I foresee that future computers will be massively parallel.

However, we are not yet fully equipped for the era of massive parallelism.
The principal challenge is managing **shared mutable states**, a key aspect of concurrency.
Coordinating multiple cores and resources requires their inputs and outputs to be synchronized through shared mutable states like memory.
Yet, managing these states is inherently difficult, both in theory and practice.
For instance, with thousands or millions of cores, how can we efficiently synchronize concurrent access to shared memory?
In the face of nondeterministic thread execution interleaving, how can we ensure the safety of a concurrent program?
And considering compiler and hardware optimizations, what constitutes the correct specification of a concurrent data structure?

Fortunately, in the past ten years, the theory of shared mutable states has made significant advances, greatly facilitating the design and analysis of practical systems utilizing these states.
Therefore, in this course, we will explore recent theories of shared mutable states and their application in real-world systems.


### Goal

This course is designed for senior undergraduate or graduate students in computer science and related disciplines who have an interest in the contemporary theory and practice of parallel computer systems.
The course aims to equip students with the ability to:

- Understand the motivations and challenges of concurrent programming.
- Learn design patterns and principles for reasoning in concurrent programming.
- Design, implement, and evaluate concurrent programs.
- Apply their knowledge to real-world parallel systems.

### Textbook

- [Slides](https://docs.google.com/presentation/d/1NMg08N1LUNDPuMxNZ-UMbdH13p8LXgMM3esbWRMowhU/edit?usp=sharing)
- References
    + [The Art of Multiprocessor Programming](https://dl.acm.org/doi/book/10.5555/2385452)
    + [The Crossbeam Library Documentation](https://docs.rs/crossbeam/latest/crossbeam/)
    + Concurrent reference counting algorithm (TBA)
    + [Behaviour-Oriented Concurrency](https://dl.acm.org/doi/10.1145/3622852)
    + [C++ Concurrency in Action](https://www.manning.com/books/c-plus-plus-concurrency-in-action-second-edition)
    + [Rust Atomics and Locks](https://marabos.nl/atomics/)

### Prerequisites

- It is **strongly recommended** that students have completed courses in:

    + Mathematics (MAS101): Propositional logic and proof techniques.
    + Data Structures (CS206): Understanding of linked lists, stacks, and queues.
    + Systems Programming (CS230) or Operating Systems (CS330): Familiarity with memory layout, caching, and locking mechanisms.
    + Programming Principles (CS220) or Programming Languages (CS320): Knowledge of lambda calculus and interpreters.

  A solid foundation in these areas is crucial for success in this course.

- Other recommended knowledge that will be beneficial:

    + Basic understanding of Computer Architecture (CS311).
    + Programming experience in [Rust](https://www.rust-lang.org/).


### Schedule

- week 1: CS230/CS330 review on concurrent programming
- week 2: Rust
- week 3: lock-based concurrency (API)
- week 4: lock-based concurrency (implementation 1)
- week 5: lock-based concurrency (implementation 2)
- week 6: lock-based concurrency (application)
- week 7: behavior-oriented concurrency (API)
- week 8: midterm exam
- week 9: lock-free concurrency (concept)
- week 10: lock-free concurrency (data structures 1)
- week 11: lock-free concurrency (data structures 2)
- week 12: lock-free concurrency (data structures 3)
- week 13: lock-free concurrency (specification)
- week 14: lock-free concurrency (garbage collection)
- week 15: behavior-oriented concurrency (implementation)
- week 16: final exam


### Tools

Ensure you are proficient with the following development tools:

- [Git](https://git-scm.com/): Essential for downloading homework templates and managing your development process.
  If you're new to Git, please complete [this tutorial](https://www.atlassian.com/git/tutorials).

    + Follow these steps to set up your repository:
        * Clone the upstream repository directly without forking it:
          ```bash
          $ git clone --origin upstream git@github.com:kaist-cp/cs431.git
          $ cd cs431
          $ git remote -v
          upstream	git@github.com:kaist-cp/cs431.git (fetch)
          upstream	git@github.com:kaist-cp/cs431.git (push)
          ```
        * To receive updates from the upstream, fetch and merge `upstream/main`:
          ```bash
          $ git fetch upstream
          $ git merge upstream/main
          ```

    + For managing your development on a Git server, create a private repository:
        * Upgrade to a "PRO" GitHub account, available at no cost.
          See the [documentation](https://education.github.com/students).
        * Configure your repository as a remote:
          ```bash
          $ git remote add origin git@github.com:<github-id>/cs431.git
          $ git remote -v
          origin	 git@github.com:<github-id>/cs431.git (fetch)
          origin	 git@github.com:<github-id>/cs431.git (push)
          upstream git@github.com:kaist-cp/cs431.git (fetch)
          upstream git@github.com:kaist-cp/cs431.git (push)
          ```
        * Push your work to your repository:
          ```bash
          $ git push -u origin main
          ```

- [Rust](https://www.rust-lang.org/): The programming language for homework assignments.
  Rust's ownership type system significantly simplifies the development of large-scale system software.

- [ChatGPT](https://chat.openai.com/) or other Large Language Models (LLMs) (optional): Useful for completing your homework.
    + In an AI-driven era, learning to effectively utilize AI in programming is crucial.
      Homework difficulty is adjusted assuming the use of ChatGPT 3.5 or an equivalent tool.

- [Visual Studio Code](https://code.visualstudio.com/) (optional): Recommended for developing your homework, although you may use any editor of your preference.

- [Single Sign On (SSO)](https://auth.fearless.systems/): Use the following SSO credentials to access [gg](https://gg.kaist.ac.kr) and the [development server](https://cloud.fearless.systems):
    + id: KAIST student id (8-digit number)
    + email: KAIST email address (@kaist.ac.kr)
    + password: Reset it here: <https://auth.fearless.systems/if/flow/default-recovery-flow/>
    + Log in to [gg](https://gg.kaist.ac.kr) using the "kaist-cp-class" option, and to the [development server](https://cloud.fearless.systems) using the "OpenID Connect" option.

- [Development Server](https://cloud.fearless.systems/):
    + **IMPORTANT: Do not attempt to hack or overload the server. Please use it responsibly.**
    + Create and connect to a workspace to use the terminal or VSCode (after installation).
    + We recommend using VSCode with the "Rust Analyzer" and "CodeLLDB" plugins.


## Grading & Honor Code

### Cheating

**IMPORTANT: READ CAREFULLY. THIS IS A SERIOUS MATTER.**

- Sign the KAIST CS Honor Code for this semester.
  Failure to do so may lead to expulsion from the course.

- We will employ sophisticated tools to detect code plagiarism.
    + Search for "code plagiarism detector" on Google Images to understand how these tools can identify advanced forms of plagiarism.
      Do not attempt plagiarism in any form.

### Programming Assignments (60%)

- All assignments will be announced at the start of the semester.
- Submit your solutions to <https://gg.kaist.ac.kr/course/19>.
- Refer to the documentation at <https://www.fearless.systems/cs431/cs431_homework/>.
- You are **permitted** to use ChatGPT or other LLMs.


### Midterm and Final Exams (40%)

- Dates & Times: April 15th (Mon), June 10th (Mon), 13:00-15:00

- Location: Room 2443, Building E3-1, KAIST

- Physical attendance is required.
  If necessary, online participation via Zoom will be accommodated.

- You are expected to bring your own laptop.
  Laptops can also be borrowed from the School of Computing Administration Team.

### Attendance (?%)

- A quiz must be completed on the [Course Management](https://gg.kaist.ac.kr/course/19) website for each session (if any).
  **Quizzes should be completed by the end of the day.**

- Failing to attend a significant number of sessions will result in an automatic grade of F.


## Communication

### Registration

- Ensure your ability to log into the [lab submission website](https://gg.kaist.ac.kr).
    + Use your `kaist-cp-class` account for login.
    + Your ID is your `@kaist.ac.kr` email address.
    + Reset your password here: [https://auth.fearless.systems/if/flow/default-recovery-flow/](https://auth.fearless.systems/if/flow/default-recovery-flow/)
    + Contact the instructor if login issues arise.

### Rules

- Course-related announcements and information will be posted on the [course website](https://github.com/kaist-cp/cs431) and the [GitHub issue tracker](https://github.com/kaist-cp/cs431/issues).
  It is expected that you read all announcements within 24 hours of their posting.
  Watching the repository is highly recommended for automatic email notifications of new announcements.

- Questions about course materials and assignments should be posted in [the course repository's issue tracker](https://github.com/kaist-cp/cs431/issues).
    + Avoid sending emails to the instructor or TAs regarding course materials and assignments.
    + Research your question using Google and Stack Overflow before posting.
    + Describe your question in detail, including:
        * Environment (OS, gcc, g++ version, and other relevant program information).
        * Used commands and their results, with logs formatted in code.
          See [this guide](https://guides.github.com/features/mastering-markdown/).
        * Any changes made to directories or files.
          For solution files, describe the modified code sections.
        * Your Google search results, including search terms and learned information.
    + Use a clear and descriptive title for your issue.
    + For further instructions, read [this section](https://github.com/kaist-cp/cs431#communication) on the course website.
    + The requirement to ask questions online first is twofold: It ensures clarity in your query and allows everyone to benefit from shared questions and answers.

- Email inquiries should be reserved for confidential or personal matters.
  Questions not adhering to this guideline (e.g., course material queries via email) will not be addressed.

- Office hours will not cover *new* questions.
  Check the issue tracker for similar questions before attending.
  If your question is not listed, post it as a new issue for discussion.
  Office hour discussions will focus on unresolved issues.

- Emails to the instructor or head TA should start with "CS431:" in the subject line, followed by a brief description.
  Include your name and student number in the email.
  Emails lacking this information (e.g., those without a student number) will not receive a response.

- If attending remotely via Zoom (https://kaist.zoom.us/my/jeehoon.kang), set your Zoom name to `<your student number> <your name>` (e.g., `20071163 강지훈`).
  Instructions for changing your Zoom name can be found [here](https://support.zoom.us/hc/en-us/articles/201363203-Customizing-your-profile).

- The course is conducted in English.
  However, you may ask questions in Korean, which will be translated into English.

## Ignore

1830eaed90e5986c75320daaf131bd3730b8575e866c4e92935a690e7c2a0717
