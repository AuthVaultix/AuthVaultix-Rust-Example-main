use AuthVaultix_rust::AuthVaultix;
use std::io;

fn main() {
    let mut AuthVaultixApp = AuthVaultix::new(
    "Teamdeveloperxd",
    "5d36476ca4",
    "4e1d8a87787f8af61c5462d12ee16e1f06d53fe314c78e985571db65f0007178",
    "1.0"
    );

    println!("Connecting...");
    AuthVaultixApp.init();

    loop {
        println!("\n[1] Login\n[2] Register\n[3] License Login\n[4] Exit");
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
                AuthVaultixApp.register(&u, &p, &l);
            }
            "3" => {
                let l = input("License: ");
                AuthVaultixApp.license_login(&l);
            }
            "4" => {
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
    let username = input("Username: ");
    let password = input("Password: ");
    (username, password)
}
