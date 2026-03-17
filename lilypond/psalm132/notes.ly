melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  fis2 e4 g4 fis2 b2 a4 g4 fis2 r2 \break

  % Line 2
  fis2 g4 fis4 e4 d4 g2 fis2 e2 r2 \break

  % Line 3
  b2 cis4 d4 b4 d4 cis2 b2 a2 r2 \break

  % Line 4
  fis2 a4 a4 b2 cis2 d4 cis4 b2 r2 \break

  % Line 5
  fis2 a2 e2 g4 fis4 a2 g2 fis1 \bar "|."
}
