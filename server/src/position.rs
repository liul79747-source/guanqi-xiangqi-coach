pub type Board = [[char; 9]; 10];
pub fn parse(fen: &str) -> Result<(Board, char), String> {
    if fen.len() > 180 || fen.contains(['\n', '\r']) { return Err("FEN 格式错误".into()); }
    let parts: Vec<_> = fen.split_whitespace().collect();
    if parts.len() < 2 || !matches!(parts[1], "w" | "b") { return Err("FEN 缺少正确的行棋方".into()); }
    let rows: Vec<_> = parts[0].split('/').collect();
    if rows.len() != 10 { return Err("FEN 必须有 10 行".into()); }
    let mut board = [[' '; 9]; 10];
    for (r, row) in rows.iter().enumerate() {
        let mut c = 0;
        for piece in row.chars() {
            if ('1'..='9').contains(&piece) { c += piece.to_digit(10).unwrap_or(0) as usize; }
            else if "rnbakcpRNBAKCP".contains(piece) && c < 9 { board[r][c] = piece; c += 1; }
            else { return Err("FEN 棋子或行长度错误".into()); }
            if c > 9 { return Err("FEN 行长度超过 9".into()); }
        }
        if c != 9 { return Err("FEN 每行必须为 9 格".into()); }
    }
    validate(&board)?;
    Ok((board, parts[1].chars().next().unwrap_or('w')))
}
pub fn validate(board: &Board) -> Result<(), String> {
    for (pieces, red) in [("KABNRCP", true), ("kabnrcp", false)] {
        for (p, limit) in pieces.chars().zip([1, 2, 2, 2, 2, 2, 5]) {
            let count = board.iter().flatten().filter(|&&v| v == p).count();
            if count > limit || (p.eq_ignore_ascii_case(&'k') && count != 1) { return Err("棋子数量或将帅识别异常，请校正棋盘".into()); }
        }
        for (r, row) in board.iter().enumerate() { for (c, &p) in row.iter().enumerate() {
            if p == ' ' || p.is_uppercase() != red { continue; }
            let kind = p.to_ascii_lowercase();
            if matches!(kind, 'k' | 'a') && (!(3..=5).contains(&c) || if red {r < 7} else {r > 2}) { return Err("将帅或士仕不在九宫内".into()); }
            if kind == 'b' && if red {r < 5} else {r > 4} { return Err("象相位置识别异常".into()); }
            if kind == 'p' && if red {r > 6} else {r < 3} { return Err("兵卒位置识别异常".into()); }
        }}
    }
    let kings: Vec<_> = board.iter().enumerate().flat_map(|(r,row)| row.iter().enumerate().filter_map(move |(c,&p)| if p.eq_ignore_ascii_case(&'k') {Some((r,c))} else {None})).collect();
    if kings.len() == 2 && kings[0].1 == kings[1].1 && (kings[0].0+1..kings[1].0).all(|r| board[r][kings[0].1] == ' ') { return Err("将帅照面，请校正棋盘".into()); }
    Ok(())
}
pub fn fen(board: &Board, side: char) -> String {
    let rows: Vec<String> = board.iter().map(|row| { let mut out=String::new(); let mut empty=0; for &p in row { if p==' ' {empty+=1;} else {if empty>0 {out.push_str(&empty.to_string());empty=0;} out.push(p);} } if empty>0 {out.push_str(&empty.to_string());} out }).collect();
    format!("{} {} - - 0 1", rows.join("/"), side)
}
pub fn valid_move(m: &str) -> bool { let b=m.as_bytes(); b.len()==4 && (b'a'..=b'i').contains(&b[0]) && b[1].is_ascii_digit() && (b'a'..=b'i').contains(&b[2]) && b[3].is_ascii_digit() }
/// Majority-vote each square across the latest screenshots. Reject the result
/// if it would require too many minority corrections or produce an invalid board.
pub fn consensus(samples: &std::collections::VecDeque<Board>) -> Option<(Board, f32)> {
    if samples.len() != 3 { return None; }
    let (a,b,c)=(&samples[0],&samples[1],&samples[2]);
    let mut board=[[' ';9];10];
    let mut agreements=0usize;
    for r in 0..10 { for col in 0..9 {
        let (x,y,z)=(a[r][col],b[r][col],c[r][col]);
        if x==y || x==z { board[r][col]=x; agreements+=if x==y && x==z {3} else {2}; }
        else if y==z { board[r][col]=y; agreements+=2; }
        else { return None; }
    }}
    let stability=agreements as f32/270.0;
    if stability<0.985 || validate(&board).is_err() { return None; }
    Some((board,stability))
}

fn between(board: &Board, r: usize, c: usize, y: usize, x: usize) -> usize {
    let dr = (y as i32 - r as i32).signum();
    let dc = (x as i32 - c as i32).signum();
    let (mut i, mut j, mut count) = (r as i32 + dr, c as i32 + dc, 0);
    while i != y as i32 || j != x as i32 {
        if board[i as usize][j as usize] != ' ' { count += 1; }
        i += dr; j += dc;
    }
    count
}

fn pseudo_legal(board: &Board, from: (usize, usize), to: (usize, usize)) -> bool {
    let (r,c) = from;
    let (y,x) = to;
    if from == to || board[r][c] == ' ' || (board[y][x] != ' ' && board[y][x].is_uppercase() == board[r][c].is_uppercase()) { return false; }
    let p = board[r][c].to_ascii_lowercase();
    let red = board[r][c].is_uppercase();
    let (dy,dx) = (y as i32-r as i32, x as i32-c as i32);
    let (ay,ax) = (dy.abs(), dx.abs());
    let palace = (3..=5).contains(&x) && if red {y>=7} else {y<=2};
    match p {
        'r' => (dx==0 || dy==0) && between(board,r,c,y,x)==0,
        'c' => (dx==0 || dy==0) && between(board,r,c,y,x)==usize::from(board[y][x]!=' '),
        'n' => (ax==2 && ay==1 && board[r][(c as i32+dx.signum()) as usize]==' ')
            || (ay==2 && ax==1 && board[(r as i32+dy.signum()) as usize][c]==' '),
        'b' => ax==2 && ay==2 && (if red {y>=5} else {y<=4}) && board[(r as i32+dy/2) as usize][(c as i32+dx/2) as usize]==' ',
        'a' => palace && ax==1 && ay==1,
        'k' => (palace && ax+ay==1) || (board[y][x].eq_ignore_ascii_case(&'k') && dx==0 && between(board,r,c,y,x)==0),
        'p' => (dx==0 && dy==if red {-1}else{1}) || (dy==0 && ax==1 && (if red {r<=4}else{r>=5})),
        _ => false,
    }
}

fn in_check(board: &Board, side: char) -> bool {
    let king = if side=='w' {'K'} else {'k'};
    let Some((kr,kc)) = board.iter().enumerate().find_map(|(r,row)| row.iter().position(|&p|p==king).map(|c|(r,c))) else {return true;};
    for r in 0..10 { for c in 0..9 {
        let p=board[r][c];
        if p!=' ' && (if p.is_uppercase() {'w'} else {'b'})!=side && pseudo_legal(board,(r,c),(kr,kc)) {return true;}
    }}
    false
}

fn legal_moves(board: &Board, side: char) -> Vec<((usize,usize),(usize,usize))> {
    let mut moves=Vec::with_capacity(64);
    for r in 0..10 {for c in 0..9 {
        let p=board[r][c];
        if p==' ' || (if p.is_uppercase() {'w'} else {'b'})!=side {continue;}
        for y in 0..10 {for x in 0..9 {
            if !pseudo_legal(board,(r,c),(y,x)){continue;}
            let mut next=*board;next[y][x]=p;next[r][c]=' ';
            if !in_check(&next,side){moves.push(((r,c),(y,x)));}
        }}
    }}
    moves
}

fn distance(a:&Board,b:&Board)->usize {
    (0..10).map(|r|(0..9).filter(|&c|a[r][c]!=b[r][c]).count()).sum()
}

fn search_plies(board:&Board,side:char,target:&Board,remaining:u8,nodes:&mut usize,seen:&mut std::collections::HashSet<(Board,char,u8)>)->bool {
    *nodes+=1;
    if board==target {return remaining==0;}
    if remaining==0 || *nodes>=60_000 || distance(board,target)>usize::from(remaining)*2 {return false;}
    let key=(*board,side,remaining);
    if !seen.insert(key){return false;}
    for ((r,c),(y,x)) in legal_moves(board,side) {
        let mut next=*board;next[y][x]=next[r][c];next[r][c]=' ';
        if search_plies(&next,if side=='w'{'b'}else{'w'},target,remaining-1,nodes,seen){return true;}
        if *nodes>=60_000 {break;}
    }
    false
}

/// Recover the next player from an exactly reachable legal position within four plies.
/// A small node budget keeps exceptional frame recovery from stalling the capture loop.
pub fn infer_plies(old:&Board,new:&Board,side_to_move:char)->Option<(char,u8)> {
    for depth in 1..=4 {
        let mut nodes=0;
        let mut seen=std::collections::HashSet::new();
        if search_plies(old,side_to_move,new,depth,&mut nodes,&mut seen){
            return Some((if depth%2==0 {side_to_move}else if side_to_move=='w' {'b'}else{'w'},depth));
        }
        if nodes>=60_000 {break;}
    }
    None
}
#[cfg(test)] mod tests {
    use super::*;
    const START: &str="rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
    #[test] fn roundtrip(){let (b,s)=parse(START).unwrap();assert_eq!(fen(&b,s),START);}
    #[test] fn reject_injection(){assert!(parse(&format!("{}\ngo infinite",START)).is_err());assert!(parse("9/9 w").is_err());}
}
