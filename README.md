# 关于本仓库

本仓库存放北京大学 2023 秋"编译原理"课程中 SysY 编译器的个人实现代码.

## 常用命令

创建容器, 开启标准输入和终端, 挂载工作目录, 运行bash, 退出后删除容器:

```
docker run -it --rm -v `pwd`:/root/compiler maxxing/compiler-dev bash
```

生成 AST:

```
cargo run -- -ast temp/hello.c -o temp/hello.ast
```

生成 Koopa 文本:

```
cargo run -- -koopa temp/hello.c -o temp/hello.koopa
```

生成 RISC-V 汇编:

```
cargo run -- -riscv temp/hello.c -o temp/hello.s
```

