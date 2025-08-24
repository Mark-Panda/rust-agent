class Game {
    constructor() {
        this.canvas = document.getElementById('game-canvas');
        this.board = new Board(this.canvas);
        this.ai = new AI(this.board);
        this.gameOver = false;
        this.currentPlayer = 1; // 1: 黑子(玩家), 2: 白子(AI)
        
        this.setupUI();
        this.startGame();
    }

    setupUI() {
        this.restartBtn = document.getElementById('restart-btn');
        this.undoBtn = document.getElementById('undo-btn');
        this.currentPlayerDisplay = document.getElementById('current-player');
        this.gameMessage = document.getElementById('game-message');

        this.restartBtn.addEventListener('click', () => this.restart());
        this.undoBtn.addEventListener('click', () => this.undo());

        // 添加键盘快捷键
        document.addEventListener('keydown', (e) => {
            if (e.key === 'r' || e.key === 'R') {
                this.restart();
            } else if (e.key === 'z' || e.key === 'Z') {
                this.undo();
            }
        });
    }

    startGame() {
        this.board.draw();
        this.updateUI();
    }

    handleMove(row, col) {
        if (this.gameOver || !this.board.isValidMove(row, col)) {
            return;
        }

        // 玩家落子
        const playerWin = this.board.makeMove(row, col, this.currentPlayer);
        
        if (playerWin) {
            this.endGame(`黑子获胜！`);
            return;
        }

        if (this.board.isDraw()) {
            this.endGame("平局！");
            return;
        }

        // AI落子
        this.currentPlayer = 2;
        this.updateUI();
        
        // 延迟AI落子，增加游戏体验
        setTimeout(() => {
            const aiWin = this.ai.makeMove();
            
            if (aiWin) {
                this.endGame(`白子(AI)获胜！`);
                return;
            }

            if (this.board.isDraw()) {
                this.endGame("平局！");
                return;
            }

            this.currentPlayer = 1;
            this.updateUI();
        }, 500);
    }

    endGame(message) {
        this.gameOver = true;
        this.gameMessage.textContent = message;
        this.gameMessage.classList.add('win-message');
        this.updateUI();
    }

    restart() {
        this.board.reset();
        this.gameOver = false;
        this.currentPlayer = 1;
        this.gameMessage.textContent = "游戏开始！黑子先行";
        this.gameMessage.classList.remove('win-message');
        this.updateUI();
    }

    undo() {
        if (this.gameOver) return;
        
        // 悔棋两次（玩家和AI各一次）
        const lastMove1 = this.board.undo(); // 悔AI的棋
        const lastMove2 = this.board.undo(); // 悔玩家的棋
        
        if (lastMove1 || lastMove2) {
            this.currentPlayer = 1;
            this.gameMessage.textContent = "游戏继续！黑子先行";
            this.gameMessage.classList.remove('win-message');
            this.gameOver = false;
            this.updateUI();
        }
    }

    updateUI() {
        this.currentPlayerDisplay.textContent = this.currentPlayer === 1 ? "黑子" : "白子(AI)";
        
        // 更新按钮状态
        this.undoBtn.disabled = this.board.moves.length === 0 || this.gameOver;
        
        // 添加游戏状态提示
        if (!this.gameOver) {
            this.gameMessage.textContent = 
                this.currentPlayer === 1 ? "轮到黑子落子" : "AI思考中...";
        }
    }
}

// 初始化游戏
document.addEventListener('DOMContentLoaded', () => {
    const game = new Game();
    
    // 将handleMove绑定到board的点击事件
    game.canvas.addEventListener('click', (e) => {
        const rect = game.canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        
        const col = Math.round(x / game.board.cellSize);
        const row = Math.round(y / game.board.cellSize);
        
        game.handleMove(row, col);
    });
    
    // 添加触摸支持
    game.canvas.addEventListener('touchstart', (e) => {
        e.preventDefault();
        const rect = game.canvas.getBoundingClientRect();
        const touch = e.touches[0];
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        
        const col = Math.round(x / game.board.cellSize);
        const row = Math.round(y / game.board.cellSize);
        
        game.handleMove(row, col);
    });
});