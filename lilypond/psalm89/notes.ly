melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 g'2 e4 d4 g4 a4 b2 b2 d4 c4 b2 a2 g2 r2 \break

  % Line 2
  g2 e4 d4 g4 a4 b2 b2 d4 c4 b2 a2 g2 r2 \break

  % Line 3
  b2 b4 a4 g4 c4 b2 g2 a4 fis4 g4 fis4 e2 d2 r2 \break

  % Line 4
  d2 e4 g4 g4 fis4 g2 d2 g4 a4 b4 g4 c2 b2 r2 \break

  % Line 5
  d2 c4 b4 a4 a4 g2 c2 b4 a4 g4 fis4 e2 d2 r2 \break

  % Line 6
  a'2 b4 a2 g2 fis4 g2 a2 b4 d4 c4 b4 a2 g1 \bar "|."
}
