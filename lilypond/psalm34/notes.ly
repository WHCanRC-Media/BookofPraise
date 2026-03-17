melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 e2 g4 fis4 g2 a2 b2 \break

  % Line 2
  b2 b4 b4 d4 cis4 b2 a2 g2 r2 \break

  % Line 3
  b2 a4 g4( fis4) a2 g2 fis4 g2 r2 \break

  % Line 4
  b2 g4 e4 a2 fis2 e2 r2 \break

  % Line 5
  e2 g4 fis4 e4 e4 d2 r2 \break

  % Line 6
  g2 fis4 g4 a2 b2 b4 ais4 b2 r2 \break

  % Line 7
  b2 d4 d4 cis2 b2 a4 g4 fis2 r2 \break

  % Line 8
  a2 g4 fis2 e2 dis4 e1 \bar "|."
}
