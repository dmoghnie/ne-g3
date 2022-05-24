

pub trait StateImpl {
    
    fn get_name(&self) -> &str;
    fn on_enter(&self, app: &App);
    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>>;
    fn on_exit(&self, app: &App);
}