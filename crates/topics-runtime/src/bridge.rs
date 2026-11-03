use serde_json::Value;
pub struct SessionGate { token: String }
impl SessionGate {
    pub fn new(_: &str, _: &str) -> Self { Self { token: "red".into() } }
    pub fn capability(&self) -> &str { &self.token }
    pub fn close(&mut self) {}
    pub fn authorize(&self,_:&str,_:&str,_:&str,_:&Value)->Result<(),String> { Ok(()) }
}
