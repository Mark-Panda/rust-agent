class AI {
    constructor(board) {
        this.board = board;
        this.player = 2; // AI使用白子
    }

    // 评估函数
    evaluatePosition(row, col, player, board) {
        let score = 0;
        const directions = [
            [0, 1], [1, 0], [1, 1], [1, -1]
        ];

        for (const [dx, dy] of directions) {
            score += this.evaluateDirection(row, col, dx, dy, player, board);
        }

        return score;
    }

    // 评估某个方向的得分
    evaluateDirection(row, col, dx, dy, player, board) {
        let score = 0;
        let count = 1;
        let blocked = 0;
        
        // 检查正向
        for (let i = 1; i < 5; i++) {
            const newRow = row + i * dx;
            const newCol = col + i * dy;
            if (newRow < 0 || newRow >= 15 || newCol < 0 || newCol >= 15) {
                blocked++;
                break;
            }
            if (board[newRow][newCol] === player) {
                count++;
            } else if (board[newRow][newCol] === 0) {
                break;
            } else {
                blocked++;
                break;
            }
        }
        
        // 检查反向
        for (let i = 1; i < 5; i++) {
            const newRow = row - i * dx;
            const newCol = col - i * dy;
            if (newRow < 0 || newRow >= 15 || newCol < 0 || newCol >= 15) {
                blocked++;
                break;
            }
            if (board[newRow][newCol] === player) {
                count++;
            } else if (board[newRow][newCol] === 0) {
                break;
            } else {
                blocked++;
                break;
            }
        }
        
        // 根据连子数和阻挡情况评分
        if (count >= 5) return 100000;
        if (count === 4 && blocked === 0) return 10000;
        if (count === 4 && blocked === 1) return 1000;
        if (count === 3 && blocked === 0) return 1000;
        if (count === 3 && blocked === 1) return 100;
        if (count === 2 && blocked === 0) return 100;
        if (count === 2 && blocked === 1) return 10;
        if (count === 1 && blocked === 0) return 10;
        
        return 0;
    }

    // 获取最佳落子
    getBestMove() {
        const emptyPositions = this.board.getEmptyPositions();
        if (emptyPositions.length === 0) return null;

        let bestScore = -Infinity;
        let bestMove = null;

        for (const {row, col} of emptyPositions) {
            // 复制棋盘
            const tempBoard = this.board.getBoard();
            tempBoard[row][col] = this.player;

            // 评估这个位置的得分
            let score = this.evaluatePosition(row, col, this.player, tempBoard);
            
            // 评估对手的威胁
            score += this.evaluatePosition(row, col, 3 - this.player, tempBoard) * 0.8;
            
            // 中心位置加分
            const centerDistance = Math.abs(row - 7) + Math.abs(col - 7);
            score += (14 - centerDistance) * 2;

            if (score > bestScore) {
                bestScore = score;
                bestMove = {row, col};
            }
        }

        return bestMove;
    }

    // 简单的AI决策
    makeMove() {
        const move = this.getBestMove();
        if (move) {
            return this.board.makeMove(move.row, move.col, this.player);
        }
        return false;
    }

    // 设置AI难度
    setDifficulty(level) {
        this.difficulty = level;
    }
}