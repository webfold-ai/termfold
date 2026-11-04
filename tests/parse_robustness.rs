use termfold::terminal::parse_event;
#[test]
fn csi_huge_number_does_not_panic() {
    let mut s = b"\x1b[".to_vec();
    s.extend(std::iter::repeat_n(b'9', 40));
    s.push(b'~');
    let _ = parse_event(&s);
}
#[test]
fn sgr_huge_number_does_not_panic() {
    let mut s = b"\x1b[<".to_vec();
    s.extend(std::iter::repeat_n(b'9', 40));
    s.extend_from_slice(b";1;1M");
    let _ = parse_event(&s);
}
#[test]
fn huge_coordinate_clamps() {
    let e = parse_event(b"\x1b[<0;99999999999;99999999999M");
    assert!(matches!(e, termfold::terminal::Event::Mouse(_)));
}
