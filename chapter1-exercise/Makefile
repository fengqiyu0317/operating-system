# Makefile for Chapter 1 Exercises
# 用于编译和测试第一章的练习题

.PHONY: all clean test-a test-b help

# 默认目标：编译所有练习
all: exercise_a_ls exercise_b_backtrace
	@echo "编译完成！"
	@echo "运行示例："
	@echo "  ./exercise_a_ls        # 练习A"
	@echo "  ./exercise_b_backtrace # 练习B"

# 练习A: 显示当前目录文件
exercise_a_ls: exercise_a_ls.rs
	@echo "编译练习A..."
	rustc exercise_a_ls.rs -o exercise_a_ls
	@echo "✓ 练习A编译完成"

# 练习B: 打印调用栈
exercise_b_backtrace: exercise_b_backtrace.rs
	@echo "编译练习B..."
	rustc exercise_b_backtrace.rs -o exercise_b_backtrace
	@echo "✓ 练习B编译完成"

# 测试练习A
test-a: exercise_a_ls
	@echo ""
	@echo "=== 测试练习A: 显示当前目录文件 ==="
	@./exercise_a_ls

# 测试练习B
test-b: exercise_b_backtrace
	@echo ""
	@echo "=== 测试练习B: 打印调用栈 ==="
	@./exercise_b_backtrace

# 运行所有测试
test: test-a test-b
	@echo ""
	@echo "所有测试完成！"

# 调试版本（带调试符号）
debug: exercise_a_ls.rs exercise_b_backtrace.rs
	@echo "编译调试版本..."
	rustc -g exercise_a_ls.rs -o exercise_a_ls_debug
	rustc -g exercise_b_backtrace.rs -o exercise_b_backtrace_debug
	@echo "✓ 调试版本编译完成"
	@echo "使用GDB调试: gdb ./exercise_b_backtrace_debug"

# 清理生成的文件
clean:
	@echo "清理生成的文件..."
	rm -f exercise_a_ls exercise_b_backtrace
	rm -f exercise_a_ls_debug exercise_b_backtrace_debug
	rm -f *.o *.d
	@echo "✓ 清理完成"

# 帮助信息
help:
	@echo "第一章练习 - Makefile 帮助"
	@echo ""
	@echo "可用目标："
	@echo "  make          - 编译所有练习"
	@echo "  make test     - 运行所有测试"
	@echo "  make test-a   - 测试练习A"
	@echo "  make test-b   - 测试练习B"
	@echo "  make debug    - 编译调试版本（带符号）"
	@echo "  make clean    - 清理生成的文件"
	@echo "  make help     - 显示此帮助信息"
	@echo ""
	@echo "练习说明："
	@echo "  练习A: 列出当前目录的文件"
	@echo "  练习B: 打印程序调用栈"
	@echo ""
	@echo "文档："
	@echo "  README.md        - 总体说明"
	@echo "  问答题解答.md    - 问答题详细解答"
	@echo "  编程题说明.md    - 编程题详细说明"
