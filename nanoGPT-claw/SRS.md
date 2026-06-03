# NanoGPT-Claw System Requirement Specification

Official SRS Document V1.0

Project Type: Lightweight CLI Multi-LLM Closed-Loop Evolution Agent
Benchmark Framework: Hermes-Agent

## 1. Project Overview

NanoGPT-Claw is a lightweight command-line native AI agent framework. The system adopts 1 Core Main LLM + Multiple Auxiliary LLM heterogeneous complementary cluster architecture.

This framework abandons redundant bloated modules of traditional agent, focuses on low resource overhead, server background daemon resident operation. System built-in complete chain-of-thought reasoning, self-introspection, two-layer persistent memory, full-cycle self-learning and autonomous iteration evolution.

NanoGPT-Claw natively integrates three major interaction terminals: Local CLI Console, Lark Feishu Bot Gateway, GitHub Original Webhook Gateway. All large models inside cluster own independent reasoning cache and exclusive memory space, every model can evolve separately. The whole system forms permanent closed-loop growth mechanism, stronger with long-term usage.

## 2. Development Objective

1. Construct lightweight Hermes-Agent level core agent capability, simplify heavy structure while retaining complete autonomous logic.
2. Use multi-model mutual verification mechanism to restrain AI hallucination, fix single model logic defects and code shortcomings.
3. Realize unified cross-terminal message scheduling for CLI, Feishu and GitHub webhook event.
4. Standard intrinsic step-by-step task decomposition, automatic internal self-check and content revision.
5. Implement full autonomous code inspection, bug self-repair, framework structural optimization, unattended system iteration.
6. High decoupling modular architecture, comply with GitHub open source warehouse standard, convenient subsequent secondary development.

## 3. Terminology Definition

- **Core Main LLM**: Global highest decision center, responsible for user intent recognition, overall task arrangement, subtask splitting, result final arbitration, whole system evolution strategy formulation.
- **Aux LLM Cluster**: Configurable 1~N auxiliary models with divided professional duties. Include code auditing, logic review, vulnerability detection, open source framework benchmark comparison, knowledge data retrieval.
- **CLI Runtime**: Basic command line running container, core program original operating carrier, support silent background running.
- **Native Gateway**: Original official API interface, no third-party forwarding middleware, guarantee data security and low delay transmission.
- **Dual Memory System**: Short-term real-time session context memory + long-term permanent evolution archive memory database.
- **Auto-Evolution Engine**: Core underlying drive for process recap, peer framework benchmarking, experience precipitation, whole system architecture upgrade.

## 4. Architecture

### 4.1 Seven-Layer Top-Down Hierarchical Structure

1. **Multi-Terminal Access Layer**: CLI terminal interactive input, Feishu robot message transceiver, GitHub webhook repository event callback monitor
2. **Unified Message Middleware Layer**: Request authority verification, data format unification, cross-end context synchronization, flow limiting, illegal request filtering, message routing distribution
3. **LLM Cluster Scheduling Layer**: Multi-model load management, asynchronous task allocation, auxiliary model result convergence, multi-viewpoint conflict judgment
4. **Deep Thinking Reasoning Layer**: Built-in complete CoT logic, progressive deduction, self-doubt introspection, secondary content correction, complex task layered split
5. **Dual Persistent Memory Layer**: Temporary conversation cache storage, historical behavior log, all gateway interaction records, optimization experience data permanent preservation
6. **Autonomous Evolution Engine Layer**: Open source agent horizontal comparison, daily execution process automatic review, capability iterative update, internal source code self-optimization
7. **Bottom Daemon Service Layer**: Process keepalive, abnormal crash automatic restart, system resource control, runtime exception tolerance, background stable residence

### 4.2 Core Architecture Advantages

Whole modules high cohesion and low coupling, each layer independent operation. Ultra-light kernel adapt 4G low-configuration cloud server 7*24 hours stable online. Three-end data fully interconnected, complete unmanned full closed-loop workflow, overall structure easy for code maintenance and version iteration.

## 5. Standard Global Workflow

1. User deliver command from any terminal, or external Github, Feishu trigger message event.
2. Unified middleware uniform encapsulate all request data, finish security filter and identity authentication.
3. Core main LLM analyze user original intention, split complex tasks into multiple independent subtasks.
4. Distribute different professional work to each auxiliary LLM for parallel reasoning processing.
5. All auxiliary models return analysis result, core model integrate all data and carry out self-inspection modification.
6. Generate finalized complete output, return data back to the original interactive gateway.
7. Record full thinking chain, operation flow and conversation content save into dual memory library.
8. Evolution engine automatically summarize all execution details, absorb optimization experience, complete one round of system capability iteration.
9. Cycle repeatedly, realize continuous cumulative self-growth forever.

## 6. Detailed Functional Requirements

### 6.1 CLI Native Core Function
1. Program main body based on pure command line development, no graphical interface, minimize dependency packages.
2. Integrate built-in CLI control instruction: service start, stop, log query, memory manage, evolution mode switch, config hot reload.
3. Real-time console print model calling status, complete thinking trace, runtime warning and error information.
4. Support offline local operation, normal reasoning work without external network environment.
5. Backend silent daemon mode, automatic hang up long-term background operation.

### 6.2 Multi-LLM Collaborative Scheduling System
1. User custom configuration file freely define 1 core main model, add or delete arbitrary quantity auxiliary models.
2. Clear fixed role division inside model cluster, professional division of labor solidified.
3. Adopt asynchronous parallel inference to promote operating efficiency, avoid task blocking.
4. Every independent LLM match exclusive memory partition, data isolated storage, separate evolution progress, not interfere with each other.
5. When multi-model output opinions conflict, main LLM execute final ruling.

### 6.3 Double-Layer Context Memory Mechanism
1. Short-term session memory: Sustain whole dialogue context coherence, record real-time thinking steps, continuous conversation linkage.
2. Long-term permanent memory: Archive all historical tasks, fault records, optimization scheme, third-party gateway all interaction data.
3. Intelligent similarity recall mechanism, automatically match historical mature solution to reduce repeated reasoning consumption.
4. Classified storage for CLI, Feishu, Github three kinds of data, independent file directory classification management.

### 6.4 Lark Feishu Native Gateway Module
1. Full docking official Feishu open platform original API, two-way message bidirectional communication.
2. Support private chat and group chat command delivery, remote control all system internal parameters.
3. Active push operating log, evolution update record, system abnormal alarm to Feishu dialogue.
4. Support document file, code attachment parsing, identify external code for reconstruction and optimization.

### 6.5 GitHub Original Webhook Gateway Core
1. Direct access GitHub official native webhook callback, zero intermediate proxy, original message capture.
2. Real-time monitor repository push, pull request, issue, branch modification, release version all repository dynamic events.
3. Agent automatically scan its own project source code, detect redundant code, structural loopholes and hidden bugs.
4. Possess independent warehouse maintenance ability: auto rewrite readme, generate iteration update log, submit code patch commit.
5. Automatically collect mainstream open-source agent framework data, import into learning library as evolution reference resources.

### 6.6 Intrinsic Self-Reflection CoT Engine
1. Force fixed reasoning flow: Complete Thinking -> Logical Deduction -> Self Examination -> Error Modify -> Final Output.
2. Active risk prediction, proactive eliminate logical loopholes and unreasonable content.
3. Complex business automatically layered disassemble, refine every execution step.
4. All thinking traces complete retention, convenient project version archive and code iteration.

### 6.7 Full Closed-Loop Self-Evolution Core
1. After every task finished, automatic full process comprehensive review.
2. Continuously benchmark Hermes-Agent and other top open-source intelligent agent frameworks, analyze structural advantages.
3. Learn excellent external framework design logic, migrate adapt to lightweight CLI architecture of NanoGPT-Claw.
4. System traverse internal kernel source code autonomously, repair latent bug, streamline redundant program structure.
5. Whole model cluster synchronous iterative upgrade, overall comprehensive capability permanent positive growth.

### 6.8 Unified Message Middleware Routing
1. Unify three-terminal message data standard format, uniform transmission protocol.
2. Request flow control, malicious access interception, external attack defense.
3. Cross-end context 100% synchronized, switch different terminal without breaking dialogue continuity.
4. Reserved universal interface extension, reserve access port for more social platform gateway in future.

## 7. Non-Functional Requirements

### 7.1 Performance Requirement
Whole framework lightweight optimization, low CPU and memory occupation. Gateway response low latency, support multi-task concurrent processing, adapt low-performance server long-term running.

### 7.2 Stability & Fault Tolerance
Built-in daemon watchdog mechanism, program crash automatic restart. LLM interface timeout, network exception automatic retry. Memory data write protection, prevent data loss from abnormal exit.

### 7.3 Scalability
All modules completely decoupled, independent code partition. Unified interface standard, convenient function expansion, easy secondary development and module plug-in access.

### 7.4 Security Specification
Sensitive key, model token, gateway secret key isolated encrypted storage. Webhook callback carry official signature verification, block illegal malicious request. Private user data separate archive, not expose in open source code.

## 8. Project Directory Tree

```
nanoGPT-claw/
├── cli_runtime/       # CLI Main Entry & Terminal Control Core
├── core_scheduler/    # Multi-LLM Cluster Dispatch Kernel
├── think_engine/      # Chain Of Thought Self-Reflection Module
├── evolve_engine/     # Auto Self-Evolution Iteration Core
├── memory_layer/      # Dual Layer Persistent Memory Storage
├── gateway_lark/      # Feishu Original Bot Gateway
├── gateway_github/    # GitHub Native Webhook Gateway
├── message_middle/    # Cross-Terminal Unified Message Middleware
├── config/            # Yaml Env Global Configuration Folder
├── daemon_service/    # Background Process Daemon Keepalive
├── logs/              # Full System Running Log Archive
├── docs/              # SRS File & Open Source Development Document
└── main.rs            # Project Startup Primary Entry
```

## 9. Version Iteration Roadmap

- **V1.0 Foundation Release**: Complete seven-layer overall architecture, three major gateway docking, multi-model scheduling logic, dual memory system, basic CoT reasoning, official open source initial release.
- **V1.5 Enhanced Version**: Optimize model collaborative algorithm, upgrade message middleware routing logic, strengthen global fault tolerance, perfect exception processing mechanism.
- **V2.0 Full Evolution Complete Version**: Unlock full closed-loop autonomous iteration, whole source code self-reconstruction, full architecture automatic upgrade, realize ultimate permanent evolution intelligent body.

## 10. Permanent Development Coding Constraints

1. All code keep lightweight principle, strictly control third-party dependency quantity, reject useless redundant code.
2. All gateway service only use official native API, prohibit any third-party transfer service.
3. Memory module, reasoning module, evolution core maintain complete decoupling forever.
4. Code annotation standard unified, comply with international GitHub open source writing specification.
5. All automatic business logic designed for long-term unattended automatic cycle operation.