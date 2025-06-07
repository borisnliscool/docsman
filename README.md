# Docsman

> __Write some docs, man!__

Docsman is a CLI tool that lets you render your Markdown docs on a simple webpage.

### About

I've noticed that when building cool projects, I always skip writing documentation. Using an external source of truth
for my documentation is generally a pain as it doesn't play nice with different branches in my repositories. Docsman
allows you to use simple markdown files inside your source code repository and have them rendered on a simple webpage.
This allows your documentation to be in the same place as your source code, makes it easy to share and collaborate with
your team, and makes it easy to find the documentation you need when you need it as you can use your IDE's search
functionality. Because Docsman compiles to a single binary, you could even add it to your version control to make it
available for the rest of your team.

### Usage

```
Usage: docsman [OPTIONS] <PATH>

Arguments:
  <PATH>  

Options:
  -p, --port <PORT>              [default: 8080]
      --host <HOST>              [default: 0.0.0.0]
  -a, --autoreload <AUTORELOAD>  [possible values: true, false]
  -l, --legend <LEGEND>          [possible values: true, false]
  -h, --help                     Print help
```