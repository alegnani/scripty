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
* ```` ~run ```<language>
<code> ``` ````

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
