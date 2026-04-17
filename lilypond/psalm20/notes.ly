melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  b'2 b4 a4 b4 d4 cis4 b2 ais4 b4 \break

  % Line 2
  g4 g4 g4 fis4 e4 d2 r2 \break

  % Line 3
  g2 a4 b4 cis4 b4 a4 g4 fis2 e4 \break

  % Line 4
  g4 fis4 e4 e4 dis4 e2 r2 \break

  % Line 5
  e2 b'4 b4 a4 a4 b4 d4 cis2 b2 r2 \break

  % Line 6
  d2 cis4 b4 a4 g4 a2 g2 r2 \break

  % Line 7
  g2 e4 fis4 g4 a4 b4 a4 g2 fis2 r2 \break

  % Line 8
  g2 a4 b4 a4 g4 fis2 e1 \bar "|."
}
