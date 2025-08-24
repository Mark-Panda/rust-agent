class Board {
    constructor(canvas, size = 15) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.size = size; // 棋盘大小 15x15
        this.cellSize = canvas.width / (size - 1);
        this.board = Array(size).fill(null).map(() => Array(size).fill(0));
        this.moves = []; // 记录落子历史
        
        this.setupCanvas();
    }

    setupCanvas() {
        this.canvas.addEventListener('click', (e) => this.handleClick(e));
    }

    // 绘制棋盘
    draw() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        
        // 绘制棋盘背景
        this.ctx.fillStyle = '#DEB887';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        
        // 绘制网格线
        this.ctx.strokeStyle = '#000';
        this.ctx.lineWidth = 1;
        
        for (let i = 0; i < this.size; i++) {
            // 横线
            this.ctx.beginPath();
            this.ctx.moveTo(0, i * this.cellSize);
            this.ctx.lineTo(this.canvas.width, i * this.cellSize);
            this.ctx.stroke();
            
            // 竖线
            this.ctx.beginPath();
            this.ctx.moveTo(i * this.cellSize, 0);
            this.ctx.lineTo(i * this.cellSize, this.canvas.height);
            this.ctx.stroke();
        }
        
        // 绘制天元和星位
        const starPoints = [3, 7, 11];
        this.ctx.fillStyle = '#000';
        starPoints.forEach(x => {
            starPoints.forEach(y => {
                this.ctx.beginPath();
                this.ctx.arc(x * this.cellSize, y * this.cellSize, 3, 0, 2 * Math.PI);
                this.ctx.fill();
            });
        });
        
        // 绘制棋子
        this.drawPieces();
    }

    // 绘制所有棋子
    drawPieces() {
        for (let row = 0; row < this.size; row++) {
            for (let col = 0; col < this.size; col++) {
                if (this.board[row][col] !== 0) {
                    this.drawPiece(row, col, this.board[row][col]);
                }
            }
        }
    }

    // 绘制单个棋子
    drawPiece(row, col, player) {
        const x = col * this.cellSize;
        const y = row * this.cellSize;
        
        this.ctx.beginPath();
        this.ctx.arc(x, y, this.cellSize * 0.4, 0, 2 * Math.PI);
        
        // 设置棋子颜色
        if (player === 1) {
            this.ctx.fillStyle = '#000';
        } else {
            this.ctx.fillStyle = '#fff';
            this.ctx.strokeStyle = '#000';
            this.ctx.lineWidth = 1;
            this.ctx.stroke();
        }
        
        this.ctx.fill();
        
        // 添加阴影效果
        this.ctx.shadowColor = 'rgba(0, 0, 0, 0.3)';
        this.ctx.shadowBlur = 3;
        this.ctx.shadowOffsetX = 2;
        this.ctx.shadowOffsetY = 2;
        this.ctx.fill();
        this.ctx.shadowColor = 'transparent';
    }

    // 处理点击事件
    handleClick(e) {
        const rect = this.canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        
        const col = Math.round(x / this.cellSize);
        const row = Math.round(y / this.cellSize);
        
        if (this.isValidMove(row, col)) {
            this.makeMove(row, col);
        }
    }

    // 检查是否为有效落子
    isValidMove(row, col) {
        return row >= 0 && row < this.size && 
               col >= 0 && col < this.size && 
               this.board[row][col] === 0;
    }

    // 落子
    makeMove(row, col, player = null) {
        if (player === null) {
            player = this.getCurrentPlayer();
        }
        
        this.board[row][col] = player;
        this.moves.push({row, col, player});
        this.draw();
        
        return this.checkWin(row, col, player);
    }

    // 获取当前玩家
    getCurrentPlayer() {
        return this.moves.length % 2 === 0 ? 1 : 2;
    }

    // 检查是否获胜
    checkWin(row, col, player) {
        const directions = [
            [0, 1],   // 水平
            [1, 0],   // 垂直
            [1, 1],   // 对角线
            [1, -1]   // 反对角线
        ];
        
        for (const [dx, dy] of directions) {
            let count = 1;
            
            // 正向检查
            for (let i = 1; i < 5; i++) {
                const newRow = row + i * dx;
                const newCol = col + i * dy;
                if (newRow >= 0 && newRow < this.size && 
                    newCol >= 0 && newCol < this.size && 
                    this.board[newRow][newCol] === player) {
                    count++;
                } else {
                    break;
                }
            }
            
            // 反向检查
            for (let i = 1; i < 5; i++) {
                const newRow = row - i * dx;
                const newCol = col - i * dy;
                if (newRow >= 0 && newRow < this.size && 
                    newCol >= 0 && newCol < this.size && 
                    this.board[newRow][newCol] === player) {
                    count++;
                } else {
                    break;
                }
            }
            
            if (count >= 5) {
                return true;
            }
        }
        
        return false;
    }

    // 检查是否平局
    isDraw() {
        return this.moves.length === this.size * this.size;
    }

    // 悔棋
    undo() {
        if (this.moves.length > 0) {
            const lastMove = this.moves.pop();
            this.board[lastMove.row][lastMove.col] = 0;
            this.draw();
            return lastMove;
        }
        return null;
    }

    // 重新开始
    reset() {
        this.board = Array(this.size).fill(null).map(() => Array(this.size).fill(0));
        this.moves = [];
        this.draw();
    }

    // 获取棋盘状态
    getBoard() {
        return this.board.map(row => [...row]);
    }

    // 获取空位
    getEmptyPositions() {
        const positions = [];
        for (let row = 0; row < this.size; row++) {
            for (let col = 0; col < this.size; col++) {
                if (this.board[row][col] === 0) {
                    positions.push({row, col});
                }
            }
        }
        return positions;
    }
}