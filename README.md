[![Rust](https://github.com/alegnani/scripty/actions/workflows/rust.yml/badge.svg)](https://github.com/alegnani/scripty/actions/workflows/rust.yml)

# Scripty
Scripty is a completely work-in-progress Discord bot which allows execution of code directly in your favourite servers.
It uses Docker to sandbox the unknown code snippets and can easily be customized to support new languages.
By running the bot's command with a Markdown code block it allows other users to more easily follow the code and the produced output.

E.g: 
````python
~run ```python
list = [i**2 for i in range(1, 11)]
print(list)
```
````

## Usage

Being only in a very early stage scripty only supports the following commands:
* `~run <CODE_BLOCK>` : Run the code in the CODE_BLOCK
* `~langs` : List the supported languages
* `~help` : Get help  

## Invite Scripty into your server

[Invite Scipty!](https://discord.com/api/oauth2/authorize?client_id=859836163769368577&permissions=84992&scope=bot)
First of all thank you for using Scripty. :)
Please note that Scripty might not be always online as it is still being developed and is not yet hosted.

## Supported languages
* C
* C++
* Python
* Rust
* Julia
* Typescript
* Javascript
* Java
* Go
