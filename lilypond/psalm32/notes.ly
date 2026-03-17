melody = \relative c'' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 d2 fis2 g2 a2 a2 a4 fis4 g4 a4 b2 a4 \break

  % Line 2
  g4 fis4 fis4 e2 a2 b4 a4 g4 fis4 e2 d2 r2 \break

  % Line 3
  d2 fis4 g4 a2 a2 b4 a4 g4 g4 fis2 r2 \break

  % Line 4
  a2 g4 fis4 e2 fis2 g4 fis4 d2 e2 d2 r2 \break

  % Line 5
  d2 d4 d4 cis2 a2 b4 cis4 d4 cis4 b2 a2 r2 \break

  % Line 6
  a2 a4 a4 b2 a2 g4 b4 a4 g4 fis2 e2 r2 r8 \break

  % Line 7
  a2 a4 a4 b2 d2 cis4 a4 b4 b4 a2 r8 \break

  % Line 8
  a2 b4 b4 a2 g2 fis4 e4 d2 e2 d1 \bar "|."
}
