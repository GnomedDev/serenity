use std::collections::HashMap;
use std::sync::Arc;

use crate::framework::standard::*;

type Group<D> = (&'static CommandGroup<D>, Arc<GroupMap<D>>, Arc<CommandMap<D>>);

#[derive(Debug)]
pub enum Map<D: 'static + Send + Sync> {
    WithPrefixes(GroupMap<D>),
    Prefixless(GroupMap<D>, CommandMap<D>),
}

pub trait ParseMap {
    type Storage;

    fn get(&self, n: &str) -> Option<Self::Storage>;
    fn min_length(&self) -> usize;
    fn max_length(&self) -> usize;
    fn is_empty(&self) -> bool;
}

#[derive(Debug, derivative::Derivative)]
#[derivative(Default(bound = ""))]
pub struct CommandMap<D: 'static + Send + Sync> {
    cmds: HashMap<String, (&'static Command<D>, Arc<CommandMap<D>>)>,
    min_length: usize,
    max_length: usize,
}

impl<D: Send + Sync + 'static> CommandMap<D> {
    pub fn new(cmds: &[&'static Command<D>], conf: &Configuration<D>) -> Self {
        let mut map = Self::default();

        for cmd in cmds {
            let sub_map = Arc::new(Self::new(cmd.options.sub_commands, conf));

            for name in cmd.options.names {
                let len = name.chars().count();
                map.min_length = std::cmp::min(len, map.min_length);
                map.max_length = std::cmp::max(len, map.max_length);

                let name =
                    if conf.case_insensitive { name.to_lowercase() } else { (*name).to_string() };

                map.cmds.insert(name, (*cmd, sub_map.clone()));
            }
        }

        map
    }
}

impl<D: Send + Sync + 'static> ParseMap for CommandMap<D> {
    type Storage = (&'static Command<D>, Arc<CommandMap<D>>);

    #[inline]
    fn min_length(&self) -> usize {
        self.min_length
    }

    #[inline]
    fn max_length(&self) -> usize {
        self.max_length
    }

    #[inline]
    fn get(&self, name: &str) -> Option<Self::Storage> {
        self.cmds.get(name).cloned()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }
}

#[derive(Debug, derivative::Derivative)]
#[derivative(Default(bound = ""))]
pub struct GroupMap<D: 'static + Send + Sync> {
    groups: HashMap<&'static str, Group<D>>,
    min_length: usize,
    max_length: usize,
}

impl<D: Send + Sync + 'static> GroupMap<D> {
    pub fn new(groups: &[&'static CommandGroup<D>], conf: &Configuration<D>) -> Self {
        let mut map = Self::default();

        for group in groups {
            let subgroups_map = Arc::new(Self::new(group.options.sub_groups, conf));
            let commands_map = Arc::new(CommandMap::new(group.options.commands, conf));

            for prefix in group.options.prefixes {
                let len = prefix.chars().count();
                map.min_length = std::cmp::min(len, map.min_length);
                map.max_length = std::cmp::max(len, map.max_length);

                map.groups.insert(*prefix, (*group, subgroups_map.clone(), commands_map.clone()));
            }
        }

        map
    }
}

impl<D: 'static + Send + Sync> ParseMap for GroupMap<D> {
    type Storage = Group<D>;

    #[inline]
    fn min_length(&self) -> usize {
        self.min_length
    }

    #[inline]
    fn max_length(&self) -> usize {
        self.max_length
    }

    #[inline]
    fn get(&self, name: &str) -> Option<Self::Storage> {
        self.groups.get(&name).cloned()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}
