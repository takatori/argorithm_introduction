pub trait Queue<T> {
    /// 値xをQueueに追加する
    fn add(&mut self, x: T);

    /// 以前に追加された「次の値」yをQueueから削除し、yを返す
    fn remove(&mut self) -> Option<T>;
}

/// 双方向キュー
/// 先頭と末尾を持った要素の列を表す
/// 先頭または末尾に要素を追加できる
pub trait Deque<T> {
    fn addFirst(&mut self, x: T);
    fn removeFirst(&mut self) -> Option<T>;
    fn addLast(&mut self, x: T);
    fn removeLast(&mut self) -> Option<T>;
}
