# 优先主题学习笔记

## A. 汇编基础速记

### 函数调用典型形态

```asm
push rbp
mov rbp, rsp
sub rsp, 0x20
; body
leave
ret
```

- `call target`：压入返回地址并跳转。
- `ret`：弹出返回地址到 RIP/EIP。
- 参数位置：
  - SysV x64：RDI, RSI, RDX, RCX, R8, R9。
  - Windows x64：RCX, RDX, R8, R9，调用者预留 shadow space。
  - x86 cdecl/stdcall：多在栈上传参，返回值 EAX。

### 条件跳转判断

- `cmp a, b` 本质计算 `a-b` 影响标志位。
- `je/jz`：ZF=1。
- `jne/jnz`：ZF=0。
- `jg/jl`：有符号比较。
- `ja/jb`：无符号比较。

### 逆向识别技巧

- 连续 `cmp + jcc`：if/else 或循环边界。
- `lea reg, [base+index*scale+disp]`：常用于地址计算，也可能是快速算术。
- 大量 `mov reg, [reg+offset]`：结构体/对象字段访问。
- 间接 `call [reg+offset]`：函数指针、虚表、导入表调用。

## B. PE 结构速记

- DOS Header：`MZ`，`e_lfanew` 指向 NT Header。
- NT Header：`PE\0\0`，包含 FileHeader 与 OptionalHeader。
- Section Table：`.text` 代码、`.rdata` 只读数据、`.data` 可写数据、`.rsrc` 资源、`.reloc` 重定位。
- Import Directory：DLL 名、函数名/序号、IAT。
- Export Directory：导出函数名、RVA、序号。
- TLS：可能包含早于入口点执行的回调，是恶意代码/保护壳常见藏点。

### PE 分析顺序

1. 查入口点与节权限，识别是否加壳（高熵、异常节名、入口在非 .text）。
2. 查导入表，快速判断能力：文件、注册表、网络、进程注入、加密。
3. 查字符串，定位错误消息、URL、命令、配置。
4. 入口点附近建立主流程，结合调试器验证。

## C. ELF 结构速记

- ELF Header：位数、端序、类型、架构、入口点。
- Program Header：装载器真正关心的段，如 `PT_LOAD`、`PT_DYNAMIC`。
- Section Header：链接/分析视角的节，如 `.text`, `.plt`, `.got`, `.dynsym`, `.rela.plt`。
- GOT/PLT：动态函数调用跳板。
- PIE + ASLR：地址随机化，调试时要区分静态偏移与运行时地址。

### ELF 分析命令

```bash
file ./a.out
readelf -h ./a.out
readelf -S ./a.out
readelf -l ./a.out
readelf -r ./a.out
objdump -d -M intel ./a.out
nm -D ./a.out
```

## D. 调试技术速记

### 断点策略

- 入口断点：理解初始化与主流程。
- API 断点：如 `CreateFile`, `ReadFile`, `strcmp`, `memcmp`, `send`, `recv`。
- 条件断点：在特定参数/地址触发，减少噪音。
- 硬件断点：监控某内存地址读写执行，适合找“谁修改了这个值”。

### 动态分析闭环

1. 静态发现可疑点：字符串、导入、比较、循环、解密函数。
2. 下断点并运行到该点。
3. 记录寄存器/栈/关键内存。
4. 改输入或状态，看路径是否变化。
5. 回到反汇编重命名变量/函数。

## E. 逆向实战报告结构

- 目标与样本：来源、hash、授权/合法性说明。
- 环境：OS、工具版本、隔离设置。
- 静态发现：导入、字符串、节、关键函数。
- 动态验证：断点、输入、观察到的状态。
- 结论：算法/协议/文件格式/漏洞根因。
- 附录：脚本、伪代码、结构体定义、截图索引。

## F. 安全与伦理边界

- 不下载或传播盗版书籍/PDF。
- 不对未授权商业软件进行破解或绕过授权。
- 恶意代码只在隔离实验环境中分析；不运行真实未知样本。
- 输出以防御、互操作、学习和研究为目的。
