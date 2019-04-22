use std::env;
use std::io::{self, stdin, Write};
use std::path::Path;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use whoami;
extern crate cicada;

fn getalias(input: &String) ->  String{
    let mut buf = String::new();
    let mut flag: bool = false;

    for c in input.chars(){
        if c == '\''{flag = !flag;}
        if flag{buf.push(c);}
    }

    buf.retain(|c| c!='\'');
    buf
}

fn main() {
    let mut envvar = HashMap::new();
    let mut aliasvar: HashMap<String,String> = HashMap::new();

    let env_sysinfo = (cicada::run("env")).stdout;
    for envline in env_sysinfo.lines() {
        let mut sep = envline.split("=");
        let var = sep.next().unwrap().to_owned();
        let val = sep.next().unwrap().to_owned();
        envvar.insert(var,val);
    }

    loop {
        if whoami::username()=="root" {
            print!("# ");
        } else {
            print!("$ ");
        }
        io::stdout().flush().expect("Flush failed!");

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let mut singleword = String::new();
        let mut stringtmp = String::new();

        for c in input.chars(){
            if c == ' ' || c=='\n'{
                stringtmp.push (' ');
                let char_vec:Vec<char> = singleword.chars().collect();
                let ch = char_vec[0];
                if ch == '$' {
                    singleword.remove(0);
                    let replace = envvar.get (&singleword);
                    if replace.is_some(){
                        singleword = replace.unwrap().to_owned();
                    }
                }
                else {
                    let replace = aliasvar.get (&singleword);
                    if replace.is_some(){
                        singleword = replace.unwrap().to_owned();
                    }
                }
                stringtmp.push_str(singleword.as_str());
                singleword = String::new();
            }else {
                singleword.push(c);
            }
        }
        input = stringtmp;

        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;
        let inputtmp = input.clone();

        if inputtmp.contains("~") || inputtmp.contains(">>") || inputtmp.contains("<<") || inputtmp.contains(">") || inputtmp.contains("<") || inputtmp.contains("&"){
            let output = cicada::run(&inputtmp);
            if output.status == 0 {
                print! ("{}",output.stdout);
            }
            else {
                print! ("{}",output.stderr);
            }
        }
        else {
            while let Some(command) = commands.next() {
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;
            match command {
                "cd" => {
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }
                    previous_command = None;
                },
                "export" => {
                    let mut argstmp = args;
                    let mut sep = argstmp.next().unwrap().split('=');
                    let var = String::from(sep.next().unwrap().to_owned());
                    let val = String::from(sep.next().unwrap().to_owned());
                    envvar.insert (var, val);
                },
                "alias" => {
                    let mut argstmp = args;
                    let mut sep = argstmp.next().unwrap().split('=');
                    let var = String::from(sep.next().unwrap().to_owned());
                    let val = getalias(&inputtmp);
                    aliasvar.insert (var, val);
                },
                "unalias" => {
                    let mut argstmp = args;
                    let vartmp = argstmp.next().unwrap().trim();
                    aliasvar.remove(vartmp);
                },
                "showalias" => {
                    let mut next:String= String::new();
                    for (key,value) in &aliasvar{
                        let order = (key.to_owned() + " = " + value + "\n").to_string();
                            next.push_str(&order);
                    }
                    let newcommand = "echo ".to_string() + &next;
                    let mut tmp = newcommand.trim().split(' ');
                    tmp.next();
                    let argsecho = tmp;

                    let stdin = previous_command
                    .map_or(
                        Stdio::inherit(),
                        |output: Child| Stdio::from(output.stdout.unwrap())
                    );

                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    let output = Command::new("echo")
                        .args(argsecho)
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
                },
                "env" => {
                    let mut next:String= String::new();
                    for (key,value) in &envvar{
                        let order = (key.to_owned() + " = " + value + "\n").to_string();
                            next.push_str(&order);
                    }
                    let newcommand = "echo ".to_string() + &next;
                    let mut tmp = newcommand.trim().split(' ');
                    tmp.next();
                    let argsecho = tmp;

                    let stdin = previous_command
                    .map_or(
                        Stdio::inherit(),
                        |output: Child| Stdio::from(output.stdout.unwrap())
                    );

                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    let output = Command::new("echo")
                        .args(argsecho)
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
                },
                "exit" => return,
                _ => {
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );

                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
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
                },
                }
            }
            if let Some(mut final_command) = previous_command {
                final_command.wait().expect ("Error");
            }
        }
    }
}
