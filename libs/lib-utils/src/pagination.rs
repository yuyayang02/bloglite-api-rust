// 使用i32而不是更合理的u32的原因是，sqlx不支持对uxx类型编解码
/// 计算分页偏移
pub fn offset(limit: i32, page: i32) -> i32 {
    (page - 1) * limit
}
