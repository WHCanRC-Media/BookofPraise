melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
   a'2 bes2 g2 f2 f2 e4 d4 f2 g2 a2 r2 \break

  % Line 2
  a2 a4 b4 c4 b4 a4 g4 f2 e2 d2 r2 \break

  % Line 3
  a'2 bes2 g2 f2 f2 e4 d4 f2 g2 a2 r2 \break

  % Line 4
  a2 a4 b4 c4 b4 a4 g4 f2 e2 d2 r2 \break

  % Line 5
  d'2 c4 b4 a4 g4 a4 b4 c2 b2 a2 r2 \break

  % Line 6
  c2 b4 a4 g2 e2 f4 g4 a4 r4 g4 f2 e2 r2 \break

  % Line 7
  a2 c4 c4 d2 d2 c4 b4 a4 g4 f2( e2) d1 \bar "|."
}
