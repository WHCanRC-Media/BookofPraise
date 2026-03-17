melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 c4 b4 a2 f2 g4 f4 e2 d2 r2 \break

  % Line 2
  f2 d4 e4 f4 g4 a4 c4 b2 a2 r2 \break

  % Line 3
  a2 bes4 a4 f4 g4 a4 bes4 g2 f2 r2 \break

  % Line 4
  f2 g4 f4 d4 e4 f4 g4 a2 d,2 r2 \break

  % Line 5
  d'2 c4 d4 a4 b4 c4 d4 b2 a2 r2 \break

  % Line 6
  a2 b2 c2 a4 f4 g4 f4 e2 d1 \bar "|."
}
