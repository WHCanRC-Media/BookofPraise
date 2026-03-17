melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 d4 d4 a'2 c2 b4 b4 a2 r2 \break

  % Line 2
  c2 b2 a2 g4 e4 f2 g2 a2 r2 \break

  % Line 3
  a2 c4 c4 d2 a2 c4 c4 b2 a2 \break

  % Line 4
  r4 e4 f2 a2 g4 e4 f4 g4 e2 d1 \bar "|."
}
