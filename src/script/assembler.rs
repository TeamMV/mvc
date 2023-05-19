use std::collections::HashMap;
use bytebuffer::ByteBuffer;
use mvutils::save::{Loader, Saver};
use mvutils::utils::{remove_quotes, format_escaped};
use crate::script::consts::*;

fn err(str: String) {
    eprintln!("{}", str);
    std::process::exit(1);
}

static mut NAMED: bool = false;

macro_rules! named_var {
    ($names:ident, $buffer:ident, $token:ident, $next:ident, $func:ident) => {
        if unsafe { NAMED } {
            let t = format!("{}_{}", $func, $token);
            if $names.contains_key(&t) {
                $buffer.push_u32(*$names.get(&t).unwrap());
            }
            else {
                $names.insert(t, *$next);
                $buffer.push_u32(*$next);
                *$next += 1;
            }
        }
        else {
            $buffer.push_u32($token.parse::<u32>().unwrap());
        }
    };
}

fn push_str_var(buffer: &mut ByteBuffer, token: &str, names: &mut HashMap<String, u32>, next_var: &mut u32, func: &str) -> u32 {
    let mut chars = token.chars();
    let ident = chars.next().unwrap();
    let str = chars.as_str();
    buffer.push_u8(ident as u8);
    let mut offset = 1;
    match ident {
        LITERAL => {
            let str = format_escaped(str);
            buffer.push_string(&str);
            offset += str.len() as u32 + 4;
        }
        VARIABLE => {
            named_var!(names, buffer, str, next_var, func);
            offset += 4;
        }
        ARGUMENT => {
            buffer.push_u32(token.parse::<u32>().unwrap());
            offset += 4;
        }
        REFERENCE => {
            named_var!(names, buffer, str, next_var, func);
            offset += 4
        }
        _ => err(format!("Invalid string identifier: {}", ident))
    }
    offset
}

fn push_val(buffer: &mut ByteBuffer, token: &str, names: &mut HashMap<String, u32>, next_var: &mut u32, func: &str) -> u32 {
    let ident = token.chars().next().unwrap();
    match ident {
        LITERAL => {
            buffer.push_u8(LITERAL as u8);
            let str = format_escaped(token.split_at(1).1);
            buffer.push_string(&str);
            str.len() as u32 + 5
        }
        VARIABLE | REFERENCE => {
            buffer.push_u8(ident as u8);
            let token = token.split_at(1).1;
            named_var!(names, buffer, token, next_var, func);
            5
        }
        ARGUMENT => {
            buffer.push_u8(ARGUMENT as u8);
            buffer.push_u32(token.parse::<u32>().unwrap());
            5
        }
        _ => {
            if token == "true" {
                buffer.push_u8(BOOLEAN_TRUE as u8);
                1
            } else if token == "false" {
                buffer.push_u8(BOOLEAN_FALSE as u8);
                1
            } else {
                if token.contains('.') {
                    buffer.push_u8(FLOAT as u8);
                    buffer.push_f64(token.parse::<f64>().unwrap());
                } else {
                    buffer.push_u8(INTEGER as u8);
                    buffer.push_i64(token.parse::<i64>().unwrap());
                }
                9
            }
        }
    }
}

fn push_prim_val(buffer: &mut ByteBuffer, token: &str, names: &mut HashMap<String, u32>, next_var: &mut u32, func: &str) -> u32 {
    let ident = token.chars().next().unwrap();
    match ident {
        LITERAL => {
            err("Argument cannot be of type string!".to_string());
            0
        }
        VARIABLE | REFERENCE => {
            buffer.push_u8(ident as u8);
            let token = token.split_at(1).1;
            named_var!(names, buffer, token, next_var, func);
            5
        }
        ARGUMENT => {
            buffer.push_u8(ARGUMENT as u8);
            buffer.push_u32(token.parse::<u32>().unwrap());
            5
        }
        _ => {
            if token == "true" {
                buffer.push_u8(BOOLEAN_TRUE as u8);
                1
            } else if token == "false" {
                buffer.push_u8(BOOLEAN_FALSE as u8);
                1
            } else {
                if token.contains('.') {
                    buffer.push_u8(FLOAT as u8);
                    buffer.push_f64(token.parse::<f64>().unwrap());
                } else {
                    buffer.push_u8(INTEGER as u8);
                    buffer.push_i64(token.parse::<i64>().unwrap());
                }
                9
            }
        }
    }
}

fn push_num_val(buffer: &mut ByteBuffer, token: &str, names: &mut HashMap<String, u32>, next_var: &mut u32, func: &str) -> u32 {
    let ident = token.chars().next().unwrap();
    match ident {
        LITERAL => {
            err("Argument cannot be of type string!".to_string());
            0
        }
        VARIABLE | REFERENCE => {
            buffer.push_u8(ident as u8);
            let token = token.split_at(1).1;
            named_var!(names, buffer, token, next_var, func);
            5
        }
        ARGUMENT => {
            buffer.push_u8(ARGUMENT as u8);
            buffer.push_u32(token.parse::<u32>().unwrap());
            5
        }
        _ => {
            if token == "true" || token == "false" {
                err("Argument cannot be of type boolean!".to_string());
                0
            } else {
                if token.contains('.') {
                    buffer.push_u8(FLOAT as u8);
                    buffer.push_f64(token.parse::<f64>().unwrap());
                } else {
                    buffer.push_u8(INTEGER as u8);
                    buffer.push_i64(token.parse::<i64>().unwrap());
                }
                9
            }
        }
    }
}

macro_rules! named {
    ($names:ident, $buffer:ident, $tokens:ident, $next:ident, $func:ident) => {
        {
            let mut offset = false;
            let mut token = $tokens.next().unwrap();
            if token.starts_with('$') {
                offset = true;
                token = token.split_at(1).1;
            }
            if unsafe { NAMED } {
                let ident = format!("{}_{}", $func, token);
                if $names.contains_key(&ident) {
                    $buffer.push_u32(*$names.get(&ident).unwrap());
                }
                else {
                    $names.insert(ident, $next);
                    $buffer.push_u32($next);
                    $next += 1;
                }
            }
            else {
                $buffer.push_u32(token.parse::<u32>().unwrap());
            }
            if offset {
                5
            }
            else {
                4
            }
        }
    };
}

pub fn jump(token: &str, index: u32, labels: &mut HashMap<String, u32>, calls: &mut Vec<u32>) -> u32 {
    if token.starts_with('-') {
        let location = index - token.split_at(1).1.parse::<u32>().unwrap();
        calls.push(location);
        calls.len() as u32 - 1
    }
    else if token.starts_with('+') {
        let location = index + token.split_at(1).1.parse::<u32>().unwrap();
        calls.push(location);
        calls.len() as u32 - 1
    }
    else if token.chars().next().unwrap().is_digit(10) {
        let location = token.parse::<u32>().unwrap();
        calls.push(location);
        calls.len() as u32 - 1
    }
    else {
        if labels.contains_key(token) {
            *labels.get(token).unwrap()
        }
        else {
            calls.push(0);
            let id = calls.len() as u32 - 1;
            labels.insert(token.to_string(), id);
            id
        }
    }
}

pub fn assemble(input: String) -> Vec<u8> {
    let input = remove_quotes(&input.trim());
    let mut buffer = ByteBuffer::new();
    let mut tokens = input.split_whitespace();
    let mut index = 4;
    let mut labels = HashMap::new();
    let mut jump_calls = Vec::new();
    let mut addresses = Vec::new();
    let mut jumps = Vec::new();
    let mut calls = Vec::new();
    let mut names: HashMap<String, u32> = HashMap::new();
    let mut next_var = 0;
    let mut func = "".to_string();
    let mut idents: HashMap<String, u32> = HashMap::new();
    let mut functions: Vec<u32> = Vec::new();
    let mut next_fn = 0;
    let mut returned = true;

    if input.starts_with(".named") {
        unsafe { NAMED = true; }
        tokens.next();
    }

    buffer.push_u32(0);

    while let Some(s) = tokens.next() {
        if s.starts_with("@") {
            if !returned {
                err("There was no return from previous function! This can lead to undefined behaviour!".to_string());
            }
            let mut ident = s.split_at(1).1;
            if ident.ends_with(':') {
                ident = ident.split_at(ident.len() - 1).0;
            }
            if !idents.contains_key(ident) {
                idents.insert(ident.to_string(), next_fn);
                next_fn += 1;
            }
            let id = idents[ident];
            if functions.len() <= id as usize {
                functions.resize(id as usize + 1, 0);
            }
            func = ident.to_string();
            functions[id as usize] = index;
            returned = false;
            continue;
        }
        else if s.starts_with(".") {
            let mut ident = s.split_at(1).1;
            if ident.ends_with(':') {
                ident = ident.split_at(ident.len() - 1).0;
            }
            if !labels.contains_key(ident) {
                jump_calls.push(addresses.len() as u32);
                labels.insert(ident.to_string(), jump_calls.len() as u32 - 1);
            }
            else {
                jump_calls[labels[ident] as usize] = addresses.len() as u32;
            }
            continue;
        }
        if func.is_empty() {
            println!("Found instructions outside a function! These will not be called unless jumped to!");
        }
        addresses.push(index);
        index += 1;
        match s.to_ascii_uppercase().as_str() {
            "NOP" => buffer.push_u8(NOOP),
            "END" => {
                buffer.push_u8(END);
                returned = true;
            }
            "MOV" => {
                buffer.push_u8(MOV);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "JMP" => {
                buffer.push_u8(JMP);
                jumps.push(buffer.get_wpos());
                buffer.push_u32(jump(tokens.next().unwrap(), addresses.len() as u32 - 1, &mut labels, &mut jump_calls));
                index += 4;
            }
            "JZ" => {
                buffer.push_u8(JZ);
                index += push_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
                jumps.push(buffer.get_wpos());
                buffer.push_u32(jump(tokens.next().unwrap(), addresses.len() as u32 - 1, &mut labels, &mut jump_calls));
                index += 4;
            }
            "CMP" => {
                buffer.push_u8(CMP);
                index += push_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
                index += push_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "JE" => {
                buffer.push_u8(JE);
                jumps.push(buffer.get_wpos());
                buffer.push_u32(jump(tokens.next().unwrap(), addresses.len() as u32 - 1, &mut labels, &mut jump_calls));
                index += 4;
            }
            "JNE" => {
                buffer.push_u8(JNE);
                jumps.push(buffer.get_wpos());
                buffer.push_u32(jump(tokens.next().unwrap(), addresses.len() as u32 - 1, &mut labels, &mut jump_calls));
                index += 4;
            }
            "JG" => {
                buffer.push_u8(JG);
                jumps.push(buffer.get_wpos());
                buffer.push_u32(jump(tokens.next().unwrap(), addresses.len() as u32 - 1, &mut labels, &mut jump_calls));
                index += 4;
            }
            "JGE" => {
                buffer.push_u8(JGE);
                jumps.push(buffer.get_wpos());
                buffer.push_u32(jump(tokens.next().unwrap(), addresses.len() as u32 - 1, &mut labels, &mut jump_calls));
                index += 4;
            }
            "JL" => {
                buffer.push_u8(JL);
                jumps.push(buffer.get_wpos());
                buffer.push_u32(jump(tokens.next().unwrap(), addresses.len() as u32 - 1, &mut labels, &mut jump_calls));
                index += 4;
            }
            "JLE" => {
                buffer.push_u8(JLE);
                jumps.push(buffer.get_wpos());
                buffer.push_u32(jump(tokens.next().unwrap(), addresses.len() as u32 - 1, &mut labels, &mut jump_calls));
                index += 4;
            }
            "CALL" => {
                buffer.push_u8(CALL);
                let call = tokens.next().unwrap();
                if BUILTIN_FUNCTIONS.contains(&call.to_ascii_uppercase().as_str()) {
                    buffer.push_u8(BUILTIN as u8);
                    buffer.push_u32(builtin(call.to_ascii_uppercase().as_str()));
                    index += 5;
                    continue;
                }
                if !idents.contains_key(call) {
                    idents.insert(call.to_string(), next_fn);
                    next_fn += 1;
                }
                let id = idents[call];
                calls.push(buffer.get_wpos());
                buffer.push_u32(id as u32);
                index += 4;
            }
            "RET" => {
                buffer.push_u8(RET);
                returned = true;
            }
            "INC" => {
                buffer.push_u8(INC);
                index += named!(names, buffer, tokens, next_var, func);
            }
            "DEC" => {
                buffer.push_u8(DEC);
                index += named!(names, buffer, tokens, next_var, func);
            }
            "ADD" => {
                buffer.push_u8(ADD);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_num_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "SUB" => {
                buffer.push_u8(SUB);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_num_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "MUL" => {
                buffer.push_u8(MUL);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_num_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "DIV" => {
                buffer.push_u8(DIV);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_num_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "MOD" => {
                buffer.push_u8(MOD);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_num_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "AND" => {
                buffer.push_u8(AND);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_prim_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "OR" => {
                buffer.push_u8(OR);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_prim_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "NOT" => {
                buffer.push_u8(NOT);
                index += named!(names, buffer, tokens, next_var, func);
            }
            "NEG" => {
                buffer.push_u8(NEG);
                index += named!(names, buffer, tokens, next_var, func);
            }
            "XOR" => {
                buffer.push_u8(XOR);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_prim_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "SHL" => {
                buffer.push_u8(SHL);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_num_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "SHR" => {
                buffer.push_u8(SHR);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_num_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "SAR" => {
                buffer.push_u8(SAR);
                index += named!(names, buffer, tokens, next_var, func);
                index += push_num_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "PUSH" => {
                buffer.push_u8(PUSH);
                index += push_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "POP" => {
                buffer.push_u8(POP);
                index += named!(names, buffer, tokens, next_var, func);
            }
            "PRINT" => {
                buffer.push_u8(PRINT);
                index += push_str_var(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "SH" => {
                buffer.push_u8(SH);
                index += push_str_var(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "PUSH_RET" => {
                buffer.push_u8(PUSH_RET);
                index += push_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            "POP_RET" => {
                buffer.push_u8(POP_RET);
                index += named!(names, buffer, tokens, next_var, func);
            }
            "CPY" => {
                buffer.push_u8(CPY);
                named!(names, buffer, tokens, next_var, func);
                index += 4;
                index += push_val(&mut buffer, tokens.next().unwrap(), &mut names, &mut next_var, &func);
            }
            _ => err(format!("Unknown instruction: {}", s)),
        }
    }

    if !idents.contains_key("main") {
        err("No main function found".to_string());
    }

    buffer.set_wpos(0);
    buffer.push_u32(functions[idents["main"] as usize]);

    for jump in jumps {
        buffer.set_rpos(jump);
        let addr = buffer.pop_u32().unwrap() as usize;
        if addr >= jump_calls.len() {
            err(format!("Invalid jump address: {}", addr));
        }
        buffer.set_wpos(jump);
        buffer.write_u32(addresses[jump_calls[addr as usize] as usize]);
    }

    for call in calls {
        buffer.set_rpos(call);
        let func = buffer.pop_u32().unwrap() as usize;
        if func >= functions.len() {
            err(format!("Invalid call address: {}", func));
        }
        buffer.set_wpos(call);
        buffer.write_u32(functions[func as usize]);
    }

    buffer.into_vec()
}

fn builtin(name: &str) -> u32 {
    match name {
        "GIT_ADD_ALL" => BUILTIN_GIT_ADD_ALL,
        "GIT_ADD" => BUILTIN_GIT_ADD,
        "GIT_COMMIT_DEFAULT" => BUILTIN_GIT_COMMIT_DEFAULT,
        "GIT_COMMIT" => BUILTIN_GIT_COMMIT,
        "GIT_PUSH_UPSTREAM" => BUILTIN_GIT_PUSH_UPSTREAM,
        "GIT_PUSH" => BUILTIN_GIT_PUSH,
        _ => BUILTIN_UNKNOWN
    }
}
