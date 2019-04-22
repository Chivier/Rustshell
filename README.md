# Position

项目位置：

[Rust shell](https://github.com/Chivier/Rustshell)

# Functions

功能介绍：

这里只是使用Rust写了一个简单的Shell，功能十分简陋，主要功能

- 简单的bash shell命令
- 简单的健壮性处理，无视多余的空格、回车、tab等字符
- 支持环境变量导出和修改
- 支持管道
- 支持基本文件重定向
- 支持alias和unalias
- 增加showalias命令输出所有的alias

# 编写流程

## 基本框架

本Shell主要参考：

- [Build a Sell in Rust](https://www.joshmcguigan.com/blog/build-your-own-shell-rust/)
- [Rust shell google](https://github.com/google/rust-shell)

基本框架由该博客提供：

```rust
fn main(){
    loop {
        print!("> ");
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // must be peekable so we know when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next()  {

            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "exit" => return,
                command => {
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => { previous_command = Some(output); },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            final_command.wait();
        }

    }
}
```

这作为基本框架。

## 框架缺陷

第一，该框架支持命令过少，很多问题无法解决

第二，cd指令不能正常使用

第三，export指令无效

第四，无法使用alias

等等……

## 环境变量处理

环境变量我采取了自己构建的方法，虽然export指令失效，但是我们的env指令可以正常执行。

构建HashMap解决

## alias解决

方法同上

## cd

利用Command库特殊指令修改目录位置，这条指令必须做在shell指令之外单独处理

## cicada扩展

实际上Rust有一个名为cicada的crate，可以执行学大多数的shell指令，但是该crate有许多未知bug，而且输入输出方法过于简化，倒置无法执行带有'\n'的指令，还是需要由自己重新构建
