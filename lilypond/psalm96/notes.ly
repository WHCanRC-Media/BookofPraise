melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d'2 d4 d4 c2 a2 c4 d4 b2 a2 r2 \break

  % Line 2
  a2 d4 d4 c2 a2 g4 f4 e2 d2 r2 \break

  % Line 3
  f2 e4 f4 g2 a2 f4 g4 a2 r2 \break

  % Line 4
  e2 f4 g4 a4 e4 g2 f2 e2 r2 \break

  % Line 5
  a2 g4 a4 d,4 e4 f4 g4 e2 d1 \bar "|."
}
