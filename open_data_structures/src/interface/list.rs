/// 値の列x(0)..x(n-1)とその列に対する操作からなる
pub trait List<T> {
    /// リストの長さnを返す
    fn size() -> usize;

    /// x(i)の値を返す
    fn get(i: usize) -> Option<T>;

    /// x(i)の値をxにする
    fn set(i: usize, x: T);

    /// xをi番目として追加し、x(i)..x(n-1)を後ろにずらす
    fn add(i: usize, x: T);

    /// x(i)を削除し、x(i+1)..x(n-1)を前にずらす
    fn remove(i: usize);
}
