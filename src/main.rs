extern crate exitcode;
extern crate tera;
use tera::Tera;
use tera::Context;
use std::collections::HashMap;
use clap::{App, Arg, ArgMatches};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use serde::{ Serialize};

fn get_parameter() -> ArgMatches {
    let matches = App::new("rmmp")
        .version("0.1.1")
        .author("Alexander Li. <superpowerlee@gmail.com>")
        .about("Map modal design file to many type of modal file")
        .arg(Arg::with_name("modal file")
            .short('m')
            .long("modal")
            .value_name("FILE")
            .help("the file path of modal design file")
            .takes_value(true)
        )
        .arg(Arg::with_name("template directory")
            .short('t')
            .long("template")
            .value_name("DICTIONARY")
            .help("the directory path of template file")
            .takes_value(true)
        )
        .arg(Arg::with_name("processor name")
            .short('p')
            .long("processor")
            .value_name("STRING")
            .help("the processor name")
            .takes_value(true)
        )
        .get_matches();
    matches
}


#[derive(Debug, Clone, Serialize)]
struct Types {
    name: String
}

#[derive(Debug, Clone, Serialize)]
struct Field {
    name: String,
    description: String,
    types: HashMap<String, Types>,
    ending: String,
}

#[derive(Debug, Clone, Serialize)]
struct Relation {
    src: String,
    tar: String,
    key: String,
}

impl Field {
    fn new(name: String, description: String) -> Self {
        Field { name, description, types: HashMap::<String, Types>::new(), ending: ",".to_string() }
    }
    fn add_type(&mut self, name: String, tp: String) {
        self.types.insert(name, Types{ name: tp});
    }
}

#[derive(Debug, Clone, Serialize)]
struct Entity {
    name: String,
    description: String,
    fields: Vec<Field>
}

impl Entity {
    fn new(name: String) -> Self {
        Entity {
            name,
            description: "".to_string(),
            fields: vec![],
        }
    }

    fn process_lines(&mut self, mut lines: Vec<&str>) {
        lines.reverse();
        let mut first_time = true;
        for line in lines{
            if line.trim().starts_with("#") {
                // 处理针对实体的说明
                self.description = format!("{}\n{}", self.description.clone(), &line[1..].to_string())
            }
            if line.trim().split(":").count() == 2 {
                let (name, desp) = line.trim().split_once(":").unwrap();
                self.fields.push(Field::new(name.to_string(), desp.to_string()));
            }
            if line.trim().starts_with("@") {
                let (tp_n, tp) = line.trim()[1..].split_once(" ").unwrap();
                let tp_name = tp_n.trim();
                let tp = tp.trim();
                if let Some(fd) = self.fields.last_mut() {
                    if first_time {
                        fd.ending = "".to_string();
                        first_time = false;
                    }
                    fd.add_type(tp_name.to_string(), tp.to_string());
                }
            }
        }
        self.fields.reverse();
    }
}


fn read_file<P: AsRef<Path>>(file_path: P) -> std::io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn process_entities(input: &str) -> Vec<Entity> {
    let mut entites: Vec<Entity> = Vec::new();
    let lines = input.split("\n").collect::<Vec<&str>>();
    let mut current_entity: Option<Entity> = None;
    let mut entity_lines: Vec<&str> = Vec::new();
    for line in lines {
        if line.ends_with(":") {
            let name = line.split(":").collect::<Vec<&str>>()[0];
            //println!("entity name: {name}");
            if current_entity.is_some() {
                if let Some(mut entity) = current_entity.clone() {
                    entity.process_lines(entity_lines.clone());
                    entity_lines = Vec::new();
                    entites.push(entity);
                }
            }
            current_entity = Some(Entity::new(name.to_string()))
        }else{
            match current_entity {
                Some(ref mut _entity)=> entity_lines.push(line),
                None=>{}
            }
        }
    }
    if current_entity.is_some() {
        if let Some(mut entity) = current_entity.clone() {
            entity.process_lines(entity_lines.clone());
            entites.push(entity);
        }
    }
    entites
}

fn get_types(entities: Vec<Entity>, cond: String, pn: String) -> String {
    let mut split_parts = cond.split(".");
    if split_parts.clone().count() < 2 {
        return "".to_string()
    }
    let entity_name = split_parts.next().unwrap()[1..].to_string();
    let field_name = split_parts.next().unwrap().to_string();
    for entity in entities.clone() {
        if entity.name.trim() == entity_name.trim() {
            for fd in entity.fields {
                if fd.name == field_name {
                    if let Some(tp) = fd.types.get(&pn) {
                        let mut split_parts = tp.name.split_whitespace();
                        return split_parts.next().unwrap().to_string()
                    }
                }
            }
        }
    }
    "".to_string()
}


fn main() {
    let config = get_parameter();
    if let Some(modal_file_path) = config.get_one::<String>("modal file") {
        let file_txt = match read_file(modal_file_path) {
            Ok(f_txt)=> f_txt,
            Err(ex)=> {
                println!("{ex}");
                std::process::exit(exitcode::DATAERR);
            }
        };
        let mut entities = process_entities(file_txt.as_str());
        let immut_entities = entities.clone();
        let mut relations: Vec<Relation> = vec![];
        for et in entities.iter_mut() {
            for fd in et.fields.iter_mut() {
                let src= et.name.clone();
                let mut tar = "".to_string();
                let mut key = "".to_string();
                let mut hash_relation = false;
                for (pn, tp) in fd.types.iter_mut() {
                    if tp.name.starts_with("@") {
                        hash_relation = true;
                        let trans = get_types(immut_entities.clone(), tp.clone().name, pn.to_string());
                        tar = tp.clone().name.as_str()[1..].split(".").next().unwrap().to_string();
                        key = fd.name.clone();
                        tp.name = trans;
                    }
                }
                if hash_relation {
                    relations.push(Relation { src, tar, key })
                }
            }
        }

        if let Some(temp_path) = config.get_one::<String>("template directory") {
            if let Some(pn) = config.get_one::<String>("processor name") {
                let temp_dir = format!("{}/*.txt", temp_path);
                let tera = match Tera::new(temp_dir.as_str()) {
                    Ok(t) => t,
                    Err(e) => {
                        println!("Parsing error(s): {}", e);
                        ::std::process::exit(1);
                    }
                };
                println!("relations: {relations:?}");
                let mut context = Context::new();
                context.insert("entities", &entities);
                context.insert("relations", &relations);
                let output_txt = match tera.render(format!("{}.txt", pn).as_str(), &context) {
                   Ok(result)=>{result},
                    Err(ex)=>{
                        println!("Parsing error: {}", ex);
                        "".to_string()
                    }
                };
                println!("{output_txt}")
            }else{
                print!("processor name must be specified!")
            }
        }else{
            print!("template dictionary path must be specified!")
        }

    }else{
        print!("modal path must be specified!")
    }
}
