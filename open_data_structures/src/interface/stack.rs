/// スタックインタフェース
/// LIFO(last-in-first-out 後入れ先だし)キューとも
pub trait Stack<T> {
    /// 値xをStackに追加する
    fn push(&mut self, x: T);

    /// 最後に追加された値yをStackから削除し、yを返す
    fn pop(&mut self) -> Option<T>;
}
