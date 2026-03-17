melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d'2 c4 b4 a2 c2 d4 c4 b2 a2 r2 \break

  % Line 2
  a2 b4 c4 d2 a2 c4 c4 b2 a2 r2 \break

  % Line 3
  a2 c2 b2 a4 c4 b4 a4 g2 f4 \break

  % Line 4
  a4 bes4 a4 g4 f4 e2 d2 r2 \break

  % Line 5
  d4 d4 a'2 a4 f4 g4 g4 a2 r2 \break

  % Line 6
  d,4 d4 a'2 a4 f4 g4 g4 a2 r2 \break

  % Line 7
  f2 g4 a4 c4 b4 a2 g2 f4 \break

  % Line 8
  bes4 a4 g4 f2 e2 d1 \bar "|."
}
