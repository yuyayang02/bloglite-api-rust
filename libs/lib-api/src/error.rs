use std::fmt::{Debug, Display};

use axum::{response::IntoResponse, Json};
use serde::Serialize;

pub trait ApiError: Display + Debug {
    fn as_error_code(&self) -> ErrorCode;
}

#[derive(Debug, Clone)]
pub enum ErrorCode {
    // ===== 认证授权领域（1xxx） =====
    /// 缺少凭证
    MissingCredentials = 1001,
    /// 无效的凭证（如用户名密码错误）
    InvalidCredentials = 1002,
    /// 权限不足（角色或权限不匹配）
    InsufficientPermissions = 1003,
    /// 令牌过期或无效
    InvalidToken = 1004,

    // ===== 资源操作领域（2xxx） =====
    /// 资源不存在（通用 404 的业务扩展）
    ResourceNotFound = 2001,
    /// 资源已存在（如重复创建）
    ResourceAlreadyExists = 2002,
    /// 资源操作冲突（如并发修改）
    ResourceConflict = 2003,

    // ===== 输入校验领域（3xxx） =====
    /// 请求参数无效（如格式错误）
    InvalidInput = 3001,
    /// 数据验证失败（如邮箱格式错误）
    DataValidationFailed = 3002,

    // ===== 系统/依赖领域 =====
    /// 内部服务错误
    InternalError = 4001,
    /// 数据库错误（如连接失败）
    DatabaseError = 4002,
    /// 外部服务错误（如调用第三方 API 失败）
    ExternalServiceError = 4003,
    /// 网络错误（如请求超时）
    NetworkError = 4004,

    // ===== 业务逻辑领域（5xxx） ===== 部分case暂不启用
    /// 操作不允许（如状态不匹配）
    OperationNotAllowed = 5001,
    /// 资源限制（如配额用完）
    ResourceLimitExceeded = 5002,
    /// 业务规则冲突（如缺少前置要求）
    BusinessRuleConflict = 5003,
    /// 依赖未满足（如缺少必要资源）
    DependencyNotSatisfied = 5004,
    /// 状态无效（如对象状态不符合预期）
    InvalidState = 5005,
    // /// 条件不满足（如前置条件未达成）
    // ConditionNotMet = 5006,
    // /// 操作超时（如业务逻辑超时）
    // OperationTimeout = 5007,
    // /// 操作冲突（如并发操作导致冲突）
    // OperationConflict = 5008,
    // /// 操作失败（如业务逻辑执行失败）
    // OperationFailed = 5009,
    // /// 操作已取消（如用户主动取消）
    // OperationCancelled = 5010,
    // /// 操作已过期（如超时失效）
    // OperationExpired = 5011,
    // /// 操作未完成（如部分成功）
    // OperationIncomplete = 5012,
    // /// 操作重复（如重复提交）
    // OperationDuplicated = 5013,
    // /// 操作无效（如参数错误）
    // OperationInvalid = 5014,
    // /// 操作不支持（如功能未实现）
    // OperationNotSupported = 5015,
    // /// 操作被拒绝（如权限不足）
    // OperationRejected = 5016,
    // /// 操作被限制（如频率限制）
    // OperationLimited = 5017,
    // /// 操作被中断（如外部信号中断）
    // OperationInterrupted = 5018,
    // /// 操作被忽略（如无需处理）
    // OperationIgnored = 5019,
    // /// 操作被跳过（如条件不满足）
    // OperationSkipped = 5020,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ApiError for ErrorCode {
    fn as_error_code(&self) -> ErrorCode {
        self.clone()
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: u32,
    pub msg: String,
    #[cfg(debug_assertions)]
    pub debug: String,
}
/// 实现了 lib_api::Error trait 的类型，都可以转为 ErrorResponse 类型
/// 而 ErrorResponse 实现了 IntoResponse
/// 所以所有实现了 lib_api::Error 的类型都可以通过into/`?`自动转为 axum Response 类型
impl<T: ApiError> From<T> for ErrorResponse {
    fn from(value: T) -> Self {
        Self {
            code: value.as_error_code() as u32,
            msg: value.to_string(),

            #[cfg(debug_assertions)]
            debug: format!("{:?}", value),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
