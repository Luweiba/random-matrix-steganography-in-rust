/// 生成随机矩阵
use rand::rngs::StdRng;
use rand::{SeedableRng, Rng, RngCore};

#[derive(Debug)]
pub struct RandMatrix {
    inner_matrix: Vec<Vec<u8>>,
    pub height: usize,
    pub width: usize,
}

impl RandMatrix {
    pub fn from_seed_u64(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut matrix = Self::get_origin_matrix(&mut rng);
        Self::row_change(&mut rng,&mut matrix, 258);
        Self::col_change(&mut matrix, 258);
        Self {
            inner_matrix: matrix,
            height: 258,
            width: 258,
        }
    }
    /// 生成9×9矩阵
    fn get_origin_matrix(rng: &mut StdRng) -> Vec<Vec<u8>> {
        let mut matrix = vec![];
        let mut num_exist = vec![false; 9];
        let mut cnt = 0;
        while cnt < 9 {
            let rand_u8: u8 = rng.gen();
            if !num_exist[(rand_u8%9) as usize] {
                cnt += 1;
                matrix.push(rand_u8%9);
                num_exist[(rand_u8%9) as usize] = true;
            }
        }
        Self::matrix_reshape(matrix, 3, 3)
    }
    /// 将1×9的向量变换为3×3的矩阵
    fn matrix_reshape(old_matrix: Vec<u8>, height: usize, width: usize) -> Vec<Vec<u8>> {
        let len = old_matrix.len();
        assert_eq!(len, height * width);
        let mut matrix = vec![];
        for row in 0..height {
            let mut row_vector = vec![];
            for col in 0..width {
                row_vector.push(old_matrix[row*height+col]);
            }
            matrix.push(row_vector);
        }
        matrix
    }
    /// 填充行 3×3 => 3×256
    fn row_change(rng: &mut StdRng, matrix: &mut Vec<Vec<u8>>, width: usize) {
        for i in 3..width {
            let mut rand_u8_3 = [0u8; 3];
            rng.fill_bytes(&mut rand_u8_3);
            let index = [0u8, 1u8, 2u8];
            let mut rand_index = rand_u8_3.iter().zip(index.iter()).map(|item| (*item.0, *item.1)).collect::<Vec<(u8, u8)>>();
            rand_index.sort_by(|a, b| a.0.cmp(&b.0));
            for j in 0..3 {
                let val = matrix[rand_index[j].1 as usize][i-3];
                matrix[j].push(val);
            }
        }
    }
    /// 填充列 3×256 => 258×256
    fn col_change(matrix: &mut Vec<Vec<u8>>, height: usize) {
        let mut rows = 3;
        while rows < height {
            for i in 0..3 {
                let row_vector = matrix[i].clone();
                matrix.push(row_vector);
            }
            rows += 3;
        }
    }
    /// 查询值
    pub fn get_val_from_random_matrix(&self, x: usize, y: usize) -> u8 {
        self.inner_matrix[x][y]
    }
    /// 隐藏搜索
    pub fn search_val(&self, x: usize, y: usize, hide_payload: u8) -> (u8, u8) {
        //println!("Enter Search!");
        let mut x_idx = (x+256-1)%256;
        let mut y_idx = (y+256-1)%256;
        if x == 255 {
            x_idx = 253;
        }
        if y == 255 {
            y_idx = 253;
        }
        if x == 0 {
            x_idx = 0;
        }
        if y == 0 {
            y_idx = 0;
        }
        for _ in 0..3 {
            for _ in 0..3 {
                //println!("x:{}, y: {}, {}, Payload: {}", x_idx, y_idx, self.inner_matrix[x_idx][y_idx], hide_payload);
                if self.inner_matrix[x_idx][y_idx] == hide_payload {
                    return (x_idx as u8, y_idx as u8);
                }
                x_idx = (x_idx + 1) % 256;
            }
            y_idx = (y_idx + 1) % 256;
        }
        (x as u8, y as u8)
    }
}