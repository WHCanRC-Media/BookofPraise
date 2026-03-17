melody = \fixed c {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  e'2 a'2 g'2 f'4 f'4 e'2 r2 \break

  % Line 2
  a'2 g'4 e'4 f'4 g'4 a'2 r2 \break

  % Line 3
  c''2 b'4 a'4 g'2 e'2 f'4 d'4 e'2 r2 \break

  % Line 4
  g'2 a'4 b'4 c''4 b'4 a'2 g'2 r8 r2 \break

  % Line 5
  c''2 b'4 d''4 a'4 c''4 b'2 a'2 r2 \break

  % Line 6
  a'2 c''2 b'2 a'4 g'4 f'4 f'4 e'1 \bar "|."
}
