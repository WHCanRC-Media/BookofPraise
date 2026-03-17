melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 fis2 a2 b4 a4 g4 e4 fis2 r2 \break

  % Line 2
  d2 a'4 a4 b2 b2 d4 d4 cis2 b2 r2 \break

  % Line 3
  fis2 b4 b4 a2 d2 d4 cis4 b2 a2 r2 \break

  % Line 4
  fis2 e4 d4 e2 fis2 g4 g4 fis2 r2 \break

  % Line 5
  a2 b2 d2 b4 g4 a4 a4 g2 fis2 r2 \break

  % Line 6
  fis2 e4 d4 g4 g4 fis2 e2 d2 r2 \break

  % Line 7
  d2 e4 g4 fis4 d4 e2 e2 fis2 r2 \break

  % Line 8
  a2 b2 a2 g4 e4 fis4 a4 g2 fis1 \bar "|."
}
