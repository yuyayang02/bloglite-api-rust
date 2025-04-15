fn main() {
    bloglite::auth::JwtState::new().generate_and_write_auth_config();
}
