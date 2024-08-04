# rmmp
A modal mapper processor by rust

this tool combine with three parts: the executable, the modal design file and the processor templates.
you can store the modal file where ever you want, either the templates.

The executable usage:
```shell
USAGE:
    rmmp [OPTIONS]

OPTIONS:
    -h, --help                     Print help information
    -m, --modal <FILE>             the file path of modal design file
    -p, --processor <STRING>       the processor name
    -t, --template <DICTIONARY>    the directory path of template file
    -V, --version                  Print version information
```
### First：make a modal design
The modal file comes with several sections, each starts with a line
```template
Entity_Name:
```
next with Field definition:
```template
    field_name: Field description
```
Before the Field definition, you can use @ to start a Processor data type mapping definition.

For example:
```template
    @postgres VARCHAR(32)
    name: the name of user
```
and you can use a line starts with # to define the description for the entity.

next is a full example modal definition:

```template
User :
    # User infomation table
    @pg varchar(32) PRIMARY KEY
    @rust String
    @typescript string
    name: name of user

    @pg int
    @rust i32
    @typescript number
    age: age of user
```

### Second: make a template
The template file is all most same as [Jinja2](https://jinja.palletsprojects.com/en/3.1.x/templates/)

next is a example for generate PostgreSQL CREATE TABLE SQLs:

```template
{% for entity in entities %}CREATE TABLE {{ entity.name }} (
    {% for fd in entity.fields %}{{fd.name}} {{ fd.types.pg.name }}{{ fd.ending }}
    {% endfor %}
);
{% endfor %}
```
save the file in templates dir, with name same as the Processor.

Here if you use file name: db_sql.txt

You can use next command:
```shell
rmmp -m path_to_modal_file -t path_to_template_dir -p db_sql
```
this will print the output content:

```shell
CREATE TABLE User  (
    name varchar(32) PRIMARY KEY,
    age int
    
);
CREATE TABLE Content  (
    title varchar(64),
    poster varchar(32)
    
);

```
If you want save the output, use output redirection operator.

```shell
rmmp -m path_to_modal_file -t path_to_template_dir -p db_sql > db.sql
```
### Draw graphics

My write a template in template_sample/dot_graph.txt.
It can generate a dot file for [Graphivz](https://graphviz.org/)

Your must install it first. In Mac:
```shell
brew install graphviz
```

You can download this file and put it in template folder.

First run with next command:

```shell
rmmp -m path_to_modal_file -t path_to_template_dir -p dot_graph > uml_pic.dot
```
then generate graphics:

```shell
dot -Tpng -o uml.png uml_pic.dot
```
then you get the PNG file


![UML](https://github.com/user-attachments/assets/3d98f3a6-206a-49f4-858c-cda3e1aaad1f)
