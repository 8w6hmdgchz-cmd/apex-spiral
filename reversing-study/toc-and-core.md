# 目录与核心知识点

> 说明：未保存非授权 PDF。以下目录基于公开书目信息与该书广泛引用的结构整理，最终页码/小节名请以合法原书为准。

## 书籍整体结构

《Reversing: Secrets of Reverse Engineering》通常分为三大部分：

1. **Reversing 101 / 基础部分**：计算机底层、操作系统、汇编、工具链、反汇编与调试基础。
2. **Applied Reversing / 应用逆向**：文件格式、软件接口、闭源库分析、兼容性和互操作性。
3. **Security-related Reversing / 安全逆向**：恶意代码、漏洞、加密/DRM/保护机制、破解与防护视角。

## 建议目录骨架

### Chapter 1. Foundations / 逆向工程基础
- 什么是逆向工程：从二进制或系统行为恢复设计、接口、算法与数据结构。
- 合法/伦理边界：互操作、安全研究、漏洞分析、恶意代码分析；避免侵犯版权、绕过授权或非法入侵。
- 逆向任务类型：静态分析、动态分析、黑盒观察、灰盒验证、符号恢复。

### Chapter 2. Low-Level Software / 底层软件基础
- CPU、寄存器、内存、栈、堆、调用约定。
- 编译、链接、装载流程。
- 机器码、汇编、反汇编之间的关系。

### Chapter 3. Windows Fundamentals / Windows 运行机制
- Win32 API、进程/线程、虚拟内存、句柄与对象。
- DLL、导入表、导出表、系统调用与用户态/内核态边界。
- Windows 程序启动、装载器行为。

### Chapter 4. Reversing Tools / 工具与方法
- 静态工具：反汇编器、反编译器、字符串/导入分析、十六进制编辑器。
- 动态工具：调试器、API Monitor、Process Monitor、抓包/日志工具。
- 基本工作流：样本分诊 → 静态定位 → 动态验证 → 重命名/注释 → 形成假设 → 证伪。

### Chapter 5. Beyond the Documentation / 文档之外的接口分析
- 分析未公开 API、私有协议、闭源库行为。
- 通过调用点、参数传播、返回值、错误路径推断接口契约。
- 构造最小复现实验验证推断。

### Chapter 6. Deciphering File Formats / 文件格式逆向
- 魔数、头部、节/块、长度字段、校验、压缩/加密特征。
- 从样本集合推断字段语义。
- 编写 parser/fuzzer 验证格式理解。

### Chapter 7. Auditing Program Binaries / 二进制审计
- 识别危险函数、边界检查缺失、整数溢出、格式化字符串、Use-after-free。
- 控制流图、数据流、污点传播思维。
- 从崩溃到根因：输入 → 状态 → 触发点 → 可利用性。

### Chapter 8. Reversing Malware / 恶意代码逆向
- 静态指标：导入、字符串、节名、熵、壳特征。
- 动态行为：持久化、注入、网络通信、C2、文件/注册表操作。
- 隔离实验环境与证据记录。

### Chapter 9. Piracy and Copy Protection / 版权保护与破解视角
- 许可证校验、序列号算法、反调试、混淆、完整性校验。
- 学习目的应聚焦防护理解与软件安全，不进行非法破解。

### Chapter 10. Antireversing Techniques / 反逆向技术
- 反调试、反虚拟机、代码混淆、控制流平坦化、加壳/解壳。
- 分析策略：绕过检测、快照对比、API hook、内存 dump、逐步去混淆。

### Appendix / 附录：x86 指令与术语
- 常见 x86 指令、标志位、调用约定、栈帧模式。

## 核心知识点总览

### 1. 汇编基础
- 寄存器：EAX/RAX、EBX/RBX、ECX/RCX、EDX/RDX、ESI/RSI、EDI/RDI、ESP/RSP、EBP/RBP、EIP/RIP。
- 栈帧：函数 prologue/epilogue、参数、局部变量、返回地址。
- 控制流：jmp/jcc/call/ret、switch jump table、循环结构。
- 数据访问：mov/lea、指针间接寻址、结构体字段偏移。
- 标志位：ZF/CF/SF/OF 对条件跳转的影响。

### 2. PE/ELF 结构
- PE：DOS Header、NT Header、Optional Header、Section Table、Import/Export、Relocation、Resource、TLS。
- ELF：ELF Header、Program Header、Section Header、.text/.data/.bss/.rodata、GOT/PLT、dynamic section、relocation。
- 装载视角：文件偏移 vs 虚拟地址、RVA/VA、权限、ASLR/PIE。

### 3. 调试技术
- 断点：软件断点、硬件断点、内存访问断点。
- 单步：step into/over/out、run until return、trace。
- 状态观察：寄存器、栈、内存、模块、线程、异常。
- 动态验证：在关键 API、比较指令、解密循环、输入解析处下断。

### 4. 逆向实战
- 从目标问题出发：找校验点、找解析器、找协议、找漏洞、找恶意行为。
- 组合方法：静态命名定位 + 动态断点验证 + 脚本化提取。
- 输出成果：伪代码、结构体定义、协议字段、漏洞根因、IoC、复现脚本。
