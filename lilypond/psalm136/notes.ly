melody = \fixed c {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 d'2 d'2 g'4 a'4 b'4 c''4 d''2 r2 \break

  % Line 2
  b'2 d''2 c''4 b'4 g'2 a'2 g'2 \break

  % Line 3
  b'2 a'2 g'4 g'4 c''4 c''4 b'2 r2 \break

  % Line 4
  g'2 a'2 fis'4 g'4 r4 e'4 e'4 d'1 \bar "|."
}
