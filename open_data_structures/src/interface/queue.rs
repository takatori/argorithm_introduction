pub trait Queue<T> {
    /// 値xをQueueに追加する
    fn enqueue(&mut self, x: T);

    /// 以前に追加された「次の値」yをQueueから削除し、yを返す
    fn dequeue(&mut self) -> Option<T>;
}
