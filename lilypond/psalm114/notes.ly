melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 d4 d4 a'2 c2 d2 d4 c4 b2 a2 r2 \break

  % Line 2
  c2 c4 b4 a2 f2 g2 g4 f4 e2 d2 r2 \break

  % Line 3
  f2 g4 a4 bes4 a4 g2 f2 r2 \break

  % Line 4
  a2 a4 a4 d,2 g2 f4 e4 d4 d4 c2 r2 \break

  % Line 5
  f2 g4 g4 a2 b2 c4 d4 c4 b4 a2 r2 \break

  % Line 6
  a2 bes4 a4 g4 f4 e2 d1 \bar "|."
}
