use scan::scan;

fn main() {
    let data = &[0xff, 0xbe, 0x01, 0x02, 0x03];
    let (a, b) = scan!("ff be %b %w", data).unwrap();
    println!("a = {a}, b = {b}");

    let c = 0x01;
    let d = u16::from_be_bytes([0x02, 0x03]);
    println!("c = {c}, d = {d}");

    assert_eq!(a, c);
    assert_eq!(b, d);
}
