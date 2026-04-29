melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c4 e4 c4 g'4 e4 c'2 \break

  % Line 2
  b4 a4 g4 f4 e4 d2 \break

  % Line 3
  d4 e4 c4 a'4 g4 fis4 \break

  % Line 4
  d4 d'4 c4 b2 a2 g2 \break

  % Line 5
  g4 a2 b2 c2 c,4 d4 e4 f4 \break

  % Line 6
  g4 a4 b4 c4 d4 c2 b2 c2 \bar "|."
}
