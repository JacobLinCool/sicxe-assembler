PROG	START   0       . start
	LDA     ONE     . ROW = 1
	STA     ROW     . |
	STA     COL     . COL = 1
	LDA     ZERO    . OFFSET = 0
	STA     OFFSET  . |
L1	LDA     ROW     . TEMP = ROW
	STA     TEMP    . |
L2	LDA     TEMP    . TEMP = TEMP * COL
	MUL     COL     . |
	LDX     OFFSET  . TABLE[OFFSET] = TEMP
	STA     TABLE,X . |
	LDA     OFFSET  . OFFSET = OFFSET + 3
	ADD     THREE   . |
	STA     OFFSET  . |
	LDA     COL     . COL = COL + 1
	ADD     ONE     . |
	STA     COL     . |
	COMP    TEN     . jump to L2 if COL < TEN
	JLT     L2      . |
	LDA     ROW     . ROW = ROW + 1
	ADD     ONE     . |
	STA     ROW     . |
	LDA     ONE     . COL = 1
	STA     COL     . |
	LDA     ROW     . jump to L1 if ROW < TEN
	COMP    TEN     . |
	JLT     L1      . |
	LDA     ONE     . DONE = 1
	STA     DONE    . |
	LDA     ZERO    . clean up all variables
	STA     ROW     . |
	STA     COL     . |
	STA     OFFSET  . |
	STA     TEMP    . |
ZERO	WORD    0       . define 0
ONE	WORD    1       . define 1
THREE	WORD    3       . define 3
TEN	WORD    10      . define 10
ROW	RESW    1       . reserve 1 word for ROW
COL	RESW    1       . reserve 1 word for COL
OFFSET	RESW    1       . reserve 1 word for OFFSET
TEMP	RESW    1       . reserve 1 word for TEMP
DONE	RESW    1       . reserve 1 word for DONE
TABLE	RESW    81      . reserve 81 (9x9) words
	END     PROG    . end
