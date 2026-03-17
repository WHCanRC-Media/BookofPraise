melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 f4 g4 a2 c2 a4 f4 g4 f4 e2 d2 r2 \break

  % Line 2
  a'2 a4 g4 a2 d2 a4 d4 c2 b2 a2 r2 \break

  % Line 3
  d,2 f4 f4 e2 d2 a'4 a4 g4 e4 f2 e2 r2 \break

  % Line 4
  a2 c4 a4 f2 g2 a4 d,4 f2 e2 d1 \bar "|."
}
