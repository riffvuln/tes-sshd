fn main() -> anyhow::Result<()> {
    let mat1 = nalgebra::Matrix3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    // calculate the lu
    let lu = mat1.lu();
    // calculate the determinant
    let det = lu.determinant();
    // print all
    println!("Matrix: \n{:?}", mat1);
    println!("LU: \n{:?}", lu);
    println!("Determinant: {:?}", det);
    Ok(())
}