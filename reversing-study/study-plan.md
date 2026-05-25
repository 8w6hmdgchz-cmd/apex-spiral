# 学习计划（按章节分解）

## 总目标

用 4 周建立软件逆向的可操作能力：能读懂常见 x86/x64 汇编、理解 PE/ELF 装载结构、熟练使用调试器验证假设，并完成小型逆向实战报告。

## 学习路径优化（A2A/自反馈版）

- 初始权重：汇编 35%，PE/ELF 25%，调试 25%，实战 15%。
- 每完成一次练习，用 1-5 分记录：理解度、复现度、卡点数量。
- 若某主题复现度 <3：下一轮增加该主题 20% 时间。
- 若连续两次理解度 ≥4：进入更复杂样本，不继续堆理论。

## Week 1：基础与汇编优先

### Day 1：逆向工程概念与环境
- 读 Chapter 1。
- 建立合法样本集：自己编译的 C 程序、开源小工具、CTF crackme（确保授权）。
- 安装/确认工具：Ghidra 或 IDA Free、x64dbg/WinDbg/lldb/gdb、radare2、objdump/readelf、PE-bear/CFF Explorer。
- 产出：`environment.md`（工具清单）与第一个 hello world 反汇编截图/笔记。

### Day 2-3：x86/x64 汇编核心
- 读 Chapter 2 与附录指令。
- 练习：编译 if/for/switch/function pointer/struct access，比较 O0/O2 反汇编。
- 重点：栈帧、调用约定、条件跳转、lea、数组/结构体偏移。
- 产出：10 个 C 片段到汇编模式对照表。

### Day 4：函数识别与控制流
- 练习在 Ghidra/IDA 中重命名函数、变量、注释基本块。
- 重点：函数边界、call graph、CFG、循环和 switch 识别。

### Day 5-7：调试入门
- 使用 gdb/lldb/x64dbg 对自己编译程序下断点。
- 练习：修改输入、观察寄存器、栈回溯、内存 patch（仅本地样本）。
- 产出：一份“静态假设 → 动态验证”的短报告。

## Week 2：PE/ELF 与装载机制

### Day 8-9：PE 文件结构
- 读 Windows fundamentals / loader 相关章节。
- 学习 DOS Header、NT Header、Section、Import/Export、RVA/VA。
- 练习：用 PE-bear 和 Python pefile 枚举导入表、节权限、入口点。

### Day 10-11：ELF 文件结构
- 学习 ELF header、program header、section header、GOT/PLT、relocation。
- 练习：`readelf -a`、`objdump -d`、`nm`、`ldd` 分析自编译 ELF。

### Day 12：动态链接与 API 追踪
- Windows：观察 LoadLibrary/GetProcAddress。
- Linux/macOS：观察 PLT/GOT、dyld/ld.so 符号解析。

### Day 13-14：文件格式逆向
- 读 Deciphering File Formats。
- 练习：设计一个 toy binary format，再从样本反推字段。
- 产出：格式结构体定义 + parser。

## Week 3：高级调试与安全逆向

### Day 15-16：二进制审计
- 读 Auditing Program Binaries。
- 练习：分析含边界错误的 toy 程序，定位崩溃根因。
- 重点：危险函数、输入长度、整数边界、内存生命周期。

### Day 17-18：恶意代码分析方法（只用安全样本/教学样本）
- 读 Malware 章节。
- 建隔离原则：虚拟机、无共享剪贴板、快照、禁止真实联网或使用模拟网络。
- 练习：分析 benign “simulated malware” 行为：自启动、文件写入、网络请求模拟。

### Day 19-20：反调试与混淆
- 读 Antireversing。
- 识别 IsDebuggerPresent、时间检测、异常控制流、字符串加密。
- 练习：对自写程序加入简单反调试，再分析检测点。

### Day 21：阶段复盘
- 汇总卡点，按理解度重新分配 Week 4 时间。

## Week 4：逆向实战项目

### Project A：闭源库接口恢复（推荐）
- 目标：给一个小型动态库恢复 3-5 个函数签名与参数语义。
- 方法：导出表/字符串/调用约定/动态调用测试。
- 产出：header 文件草案 + 行为测试。

### Project B：文件格式恢复
- 目标：从 5-10 个样本恢复格式字段并写 parser。
- 产出：格式文档 + parser + 异常样本处理。

### Project C：crackme 教学样本分析（合法授权）
- 目标：定位输入校验逻辑并写 keygen 思路说明。
- 边界：只用于授权教学样本，不用于商业软件破解。
- 产出：算法伪代码 + 调试证据。

## 每章学习模板

1. 章节目标：本章解决什么逆向问题？
2. 新概念：最多 10 条。
3. 关键工具命令：最多 10 条。
4. 最小练习：能复现的 toy example。
5. 卡点：哪里不确定？下一轮如何验证？
6. 产出：截图、伪代码、结构体、脚本或报告。
