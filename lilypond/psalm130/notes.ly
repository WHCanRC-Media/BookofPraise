melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 d,4 e4 f4 e4 d2 c2 r2 \break

  % Line 2
  f2 d4 e4 f2 g2 a2 r2 \break

  % Line 3
  a2 g4 a4 b4 c4 b2 a2 r2 \break

  % Line 4
  d2 c4 a4 c2 b2 a2 r2 \break

  % Line 5
  a2 c2 a2 g4 f4 e2 d2 r2 \break

  % Line 6
  g2 f4 e4 d4 d4 c2 r2 \break

  % Line 7
  f2 f4 g4 a4 bes4 g2 f2 r2 \break

  % Line 8
  a2 g4 f4 g2 e2 d1 \bar "|."
}
