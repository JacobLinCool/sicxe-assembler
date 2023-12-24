PROG	START   0       . start
	LDX     #0      . OFFSET = 0
	LDS     #0      . ROW = 0
L1	LDA     #1      . ROW = ROW + 1
	ADDR    A,S     . |
	LDT     #0      . COL = 0
L2	LDA     #1      . COL = COL + 1
	ADDR    A,T     . |
	MULR    S,A     . RESULT = RESULT * ROW
	MULR	T,A     . RESULT = RESULT * COL
	STA     TABLE,X . |
	LDA     #0      . OFFSET = OFFSET + 3
	ADDR    X,A     . |
	ADD     #3      . |
	LDX     #0      . |
	ADDR    A,X     . |
	LDA     #0      . jump to L2 if COL < 9
	ADDR	T,A     . |
	COMP    #9      . |
	JLT     L2      . |
	LDA     #0      . jump to L1 if ROW < 9
	ADDR	S,A     . |
	COMP    #9      . |
	JLT     L1      . |
TABLE	RESW    81      . reserve 81 (9x9) words
	END     PROG    . end
