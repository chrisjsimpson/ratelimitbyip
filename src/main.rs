use ratelimit::Ratelimiter;
//use std::time::Duration;
use std::io;
//use std::io::{self, stdin, Read, Write};
use std::process;
use std::time::Duration;
use std::net::Ipv4Addr;
use std::collections::HashMap;

/*

What I want this program to do

- Take an IP address from stdin
- Store it somewhere (in a list- since there will be others)
    - and create a rate limit bucket for that ip address
    - Keep Expiring IP addresses in that list after x time
        - store the "tic" timestamp of when request came in (important?)
        - When identifying ip addresses & buckets to clear, compare the stored "tic"
        with "toc" (time now) (more timers needed?) 
- When the ip address is seen, check it's rate limit
- Remember we use stderror for information messages because
  apache reads from the stdout of our program as the response.

Q: More context please
A: Apache lets you call an an executable program for rewrite purposes.

   > "This program is started once, when the Apache HTTP Server is started,
      and then communicates with the rewriting engine via STDIN and STDOUT."
   (https://httpd.apache.org/docs/2.4/rewrite/rewritemap.html#prg:~:text=prg%3A-,External%20Rewriting%20Program,-When%20a%20MapType)
   
   I'm using that crevis as a (hacky) way in enable a 'simple' per-client ip rate limmiter.

Q: Why does this print to stderr so much?
A: Because apache binds to stdout of this program

Q: How do I run and test this?
A: 
  # Make a FIFOs (named pipe):
  mkfifo /tmp/myfifo; 
  # Then start program with stdin of pipe:
  cargo run < /tmp/myfifo
  # In a new terminal, send an IP address (currently IPv4 only)
  # to the stdin of that named pipe
  echo "192.168.0.1" > /tmp/myfifo
  # Observe the output of the program, keep hitting until you're
  # rate limited.

*/


fn main() -> io::Result<()> {
    println!("Hello, world!");
    let mypid = process::id();
    eprintln!("My pid is {}", mypid);
    eprintln!("Don't Test me with echo '127.0.0.1' > /proc/{}/fd/0", mypid);
    eprintln!("instead, create a fifo file and start me with stdin redirected to me");
    eprintln!("e.g. mkfifo /tmp/myfifo; # Then start program with stdin of pipe:");
    eprintln!("./ratebyip < /tmp/myfifo");
    eprintln!("Then you can test by echo'ing to the program in another terminal");
    eprintln!("echo 192.168.1.1 > /tmp/myfifo");


    // Create hashmap to store the IP addresses and
    // rate limiters
    let mut clients:HashMap<Ipv4Addr, Ratelimiter>  = HashMap::new();
    eprintln!("clients hashmap created.");


    loop {
      // Read in ip address          
      let mut line = String::new();
      let n_bytes_read = io::stdin().read_line(&mut line);
      match n_bytes_read {
          Ok(n_bytes_read) => {
            if n_bytes_read > 0 {
              eprintln!("n_bytes_read is > 0");
              eprintln!("Read {:?} bytes.", n_bytes_read);
              eprintln!("The stdin input was: {}", line);
              // Try to parse input as Ipv4Addr
              let client_ip: Result<_, _> = line.trim().parse::<Ipv4Addr>();
              match client_ip {
                  Ok(client_ip) => {
                    eprintln!("Parsed input as ipv4 address: {:?}", client_ip);
                    let mut process_client_ip = || {
                      eprintln!("Processing client_ip: {}", client_ip);
                      eprintln!("Checking if client_ip is alread in HashMap (if not will add it): {}", client_ip);
                      if !clients.contains_key(&client_ip) {
                        eprintln!("client_ip {} not found in clients HashMap, adding it", client_ip);
                        eprintln!("Creating rate limiter for client_ip: {}", client_ip);
                        let ratelimiter = Ratelimiter::builder(1, Duration::from_secs(5))
                        .max_tokens(1)
                        .initial_available(5)
                        .build()
                        .unwrap();
                        eprintln!("Inserted new client and ratelimiter into clients HashMap. {}", client_ip);
                        clients.insert(client_ip, ratelimiter);

                      } else {
                        eprintln!("client_ip {} was found in clients HashMap.", client_ip);
                        eprintln!("The length of the clients HashMap is {}", clients.len());
                      }

                      eprintln!("Checking if client_ip has exceeded rate limit");

                      match clients.get(&client_ip) {
                        Some(client_rate_limiter ) => { 
                          eprintln!("Got rate limiter out for {}", client_ip);
                          match client_rate_limiter.try_wait() {
                              Ok(_) => eprintln!("Proceed! {} Not rate limited yet!", client_ip),
                              Err(duration  )  => eprintln!("rate limmited! {:?}", duration)
                          }
                      },
                        None => eprintln!("Could not locate rate limiter for client ip {}", client_ip)
                      }
                      
                    };

                    process_client_ip();
    
                  }
                  Err(e) => eprintln!("error parsing Ipv4Addr: {e:?}"),
              }
            }
          },
          Err(_) => ()
      }
      //thread::sleep(Duration::from_secs(1));
      //thread::yield_now(); // TODO wait for stdin rather than sleeping
    }


    // Use the rate limiter
    // loop {
    //     // a simple sleep-wait
    //     if let Err(sleep) = ratelimiter.try_wait() {
    //         std::thread::sleep(sleep);
    //         println!("Allowed");
    //         continue;
    //     }

    //     // do some rate limited action here
    //     println!("Rate limited!")
    // }
}
