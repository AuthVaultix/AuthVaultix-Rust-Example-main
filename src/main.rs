use AuthVaultix_rust::AuthVaultix;
use std::io;

fn main() {
    let mut AuthVaultixApp = AuthVaultix::new(
        "", // appname
        "", // ownerid
        "", // secret
        "1.0" // version
    );

    println!("Connecting...");
    AuthVaultixApp.init();

    loop {
        println!("\n[1] Login\n[2] Register\n[3] License Login\n[4] Upgrade\n[5] Forgot Password\n[6] Exit");
        print!("Choose option: ");
        io::Write::flush(&mut io::stdout()).unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                let (u, p) = input_credentials();
                AuthVaultixApp.login(&u, &p);
            }
            "2" => {
                let (u, p) = input_credentials();
                let l = input("License: ");
                AuthVaultixApp.register(&u, &p, &l, "");
            }
            "3" => {
                let l = input("License: ");
                AuthVaultixApp.license_login(&l);
            }
            "4" => {
                let u = input("Username: ");
                let l = input("License: ");
                AuthVaultixApp.upgrade(&u, &l);
            }
            "5" => {
                let u = input("Username: ");
                let e = input("Email: ");
                AuthVaultixApp.forgot_password(&u, &e);
            }
            "6" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid option!"),
        }
    }
}

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::Write::flush(&mut io::stdout()).unwrap();
    let mut val = String::new();
    io::stdin().read_line(&mut val).unwrap();
    val.trim().to_string()
}

fn input_credentials() -> (String, String) {
    let u = input("Username: ");
    let p = input("Password: ");
    (u, p)
}
