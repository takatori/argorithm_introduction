pub trait Queue<T> {
    /// 値xをQueueに追加する
    fn add(&mut self, x: T);

    /// 以前に追加された「次の値」yをQueueから削除し、yを返す
    fn remove(&mut self) -> Option<T>;
}
