melody = \fixed c {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 d'2 d'4 d'4 f'4 f'4 g'4 g'4 a'2 r2 \break

  % Line 2
  a'2 b'4 c''4 a'4 d''2 c''4 b'2 a'2 r2 \break

  % Line 3
  a'2 g'4 f'4 e'4 f'4 g'4 a'4 g'2 f'2 r2 \break

  % Line 4
  f'2 g'4 g'4 a'2 c''2 c''4 b'4 a'2( f'2) e'2 \break

  % Line 5
  a'2 g'4 e'4 g'2 f'4 d'4 e'2 d'1 \bar "|."
}
