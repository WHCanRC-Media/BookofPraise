melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 d4 d4 e2 g2 fis4 g4 b2 a2 g2 r2 \break

  % Line 2
  g2 g4 fis4 g4 a4 b4 a4 g2 fis2 e2 r2 \break

  % Line 3
  g2 e4 fis4 d2 e2 fis4 a2 g2 fis4 g2 r2 \break

  % Line 4
  d2 g4 a4 b2 a2 g4 fis4 e4 e4 d2 r2 \break

  % Line 5
  d2 e4 fis4 g2 d2 g4 fis4 g4 a4 b2 r2 \break

  % Line 6
  b2 d4 c4 b2 a2 g4 c4 b2 a2 g2 r2 \break

  % Line 7
  g2 a2 b2 g4 g4 c4 b4 g2 a2 b2 r2 \break

  % Line 8
  b2 a4 a4 g2 e2 fis4 g4 e4 e4 d1 \bar "|."
}
