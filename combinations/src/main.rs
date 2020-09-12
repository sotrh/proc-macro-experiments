use combinations::*;

minimal!(mini3, 3);
punctuated!(punct4, 4);
combinations!(comb2, 2);

fn main() {
    println!("{}", mini3());
    println!("{:?}", punct4());

    let input = &[0, 1];
    println!("{:?}", comb2(input));
}