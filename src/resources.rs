use std::any::{TypeId, Any};

use fxhash::FxHashMap;


#[derive(Default)]
pub struct ResourceManager{
    resources: FxHashMap<TypeId, Box<dyn Any>>,
}

// TODO: Find a way to limit to pipelines.
impl ResourceManager{
    pub fn insert<C: 'static>(&mut self, res: C){
        let ty_id = TypeId::of::<C>();
        self.resources.insert(ty_id, Box::new(res));
    }
    pub fn get<C: 'static>(&self) -> Option<&C>{
        let ty_id = TypeId::of::<C>();
        self.resources.get(&ty_id)?.downcast_ref::<C>()
    }
    pub fn has<C: 'static>(&self) -> bool{
        let ty_id = TypeId::of::<C>();
        self.resources.contains_key(&ty_id)
    }

    fn get_or<C: 'static, F: FnMut(&mut Self) -> C>(&mut self, mut f: F) -> &C{
        if self.has::<C>(){
            self.get().unwrap()
        }
        else{
            let res = f(self);
            self.insert(res);
            self.get().unwrap()
        }
    }

    fn get_or_insert<C: 'static>(&mut self, res: C) -> &C{
        if self.has::<C>(){
            self.get().unwrap()
        }
        else{
            self.insert(res);
            self.get().unwrap()
        }
    }

    pub fn get_mut<C: 'static>(&mut self) -> Option<&mut C>{
        let ty_id = TypeId::of::<C>();
        self.resources.get_mut(&ty_id)?.downcast_mut::<C>()
    }

    fn get_mut_or<C: 'static, F: FnMut(&mut Self) -> C>(&mut self, mut f: F) -> &C{
        if self.has::<C>(){
            self.get_mut().unwrap()
        }
        else{
            let res = f(self);
            self.insert(res);
            self.get_mut().unwrap()
        }
    }

    fn get_mut_or_insert<C: 'static>(&mut self, res: C) -> &mut C{
        if self.has::<C>(){
            self.get_mut().unwrap()
        }
        else{
            self.insert(res);
            self.get_mut().unwrap()
        }
    }
}

pub trait StaticResourceManager{
    fn insert<C: 'static>(&mut self, res: C);
    fn get<C: 'static>(&self) -> Option<&C>;
    fn get_mut<C: 'static>(&mut self) -> Option<&mut C>;
}

impl StaticResourceManager for Option<ResourceManager>{
    fn insert<C: 'static>(&mut self, res: C) {
        match self{
            Some(res_manager) => res_manager.insert(res),
            None => {
                let mut res_manager = ResourceManager::default();
                res_manager.insert(res);
                *self = Some(res_manager);
            }
        }
    }

    fn get<C: 'static>(&self) -> Option<&C> {
        match self{
            Some(res_manager) => res_manager.get(),
            None => None,
        }
    }

    fn get_mut<C: 'static>(&mut self) -> Option<&mut C> {
        match self{
            Some(res_manager) => res_manager.get_mut(),
            None => None,
        }
    }
}
